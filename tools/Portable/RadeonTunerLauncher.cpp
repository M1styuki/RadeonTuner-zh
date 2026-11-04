// Minimal native Win32 launcher for the single-file RadeonTuner delivery.
// The application payload is embedded as an uncompressed-by-launcher CAB resource and is
// extracted with the Windows Cabinet/FDI API. The launcher itself uses the static CRT (/MT)
// and exits immediately after starting the core executable in its per-user App directory.

#ifndef WIN32_LEAN_AND_MEAN
#define WIN32_LEAN_AND_MEAN
#endif
#ifndef NOMINMAX
#define NOMINMAX
#endif

#include <windows.h>
#include <shlobj.h>
#include <shellapi.h>
#include <bcrypt.h>
#include <fdi.h>
#include <tlhelp32.h>

#include <algorithm>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <cwchar>
#include <iterator>
#include <map>
#include <new>
#include <set>
#include <string>
#include <vector>

#include "buildconfig.h"

#pragma comment(lib, "cabinet.lib")
#pragma comment(lib, "shell32.lib")
#pragma comment(lib, "bcrypt.lib")
#pragma comment(lib, "user32.lib")

namespace
{
	constexpr int kPayloadResourceId = 101;
	constexpr wchar_t kCoreName[] = L"RadeonTuner.exe";
	constexpr wchar_t kManifestName[] = L".radeontuner-payload.txt";
	constexpr wchar_t kStateName[] = L".radeontuner-payload.state";
	constexpr wchar_t kPendingName[] = L".radeontuner-payload.pending";
	constexpr wchar_t kUpdaterConfigName[] = L"Settings\\Updater.json";
	constexpr wchar_t kMainWindowClass[] = L"RadeonTunerMainWindow";
	constexpr wchar_t kManifestHeader[] = L"RTPAYLOAD 1";
	constexpr wchar_t kStateHeader[] = L"RTPORTABLE 1";
	constexpr size_t kMaxManifestBytes = 1024 * 1024;
	constexpr size_t kMaxStateBytes = 16 * 1024;

	struct PayloadFile
	{
		std::wstring relativePath;
		std::wstring sha256;
		unsigned long long size = 0;
	};

	struct PayloadManifest
	{
		std::vector<PayloadFile> files;
		std::wstring coreSha256;
	};

	struct PortableState
	{
		bool valid = false;
		bool external = false;
		std::wstring payload;
		std::wstring coreSha256;
	};

	struct EmbeddedCab
	{
		const BYTE* data = nullptr;
		DWORD size = 0;
	};

	struct CabReadHandle
	{
		DWORD position = 0;
	};

	EmbeddedCab g_embeddedCab;
	std::wstring g_stageRoot;
	std::string g_extractError;
	std::string g_fdiTrace;
	std::set<std::wstring> g_extractedNames;
	std::set<INT_PTR> g_cabReadHandles;

	void AppendFdiTrace(const std::string& line)
	{
		if (g_fdiTrace.size() < 64 * 1024)
		{
			g_fdiTrace += line;
			g_fdiTrace += "\n";
		}
	}

	// -------------------------------------------------------------------------
	// Small string / path helpers
	// -------------------------------------------------------------------------

	std::wstring Trim(const std::wstring& value)
	{
		size_t first = 0;
		while (first < value.size() && (value[first] == L' ' || value[first] == L'\t' || value[first] == L'\r' || value[first] == L'\n'))
		{
			first++;
		}
		size_t last = value.size();
		while (last > first && (value[last - 1] == L' ' || value[last - 1] == L'\t' || value[last - 1] == L'\r' || value[last - 1] == L'\n'))
		{
			last--;
		}
		return value.substr(first, last - first);
	}

	std::wstring ToLowerAscii(const std::wstring& value)
	{
		std::wstring result = value;
		for (wchar_t& character : result)
		{
			if (character >= L'A' && character <= L'Z')
			{
				character = (wchar_t)(character - L'A' + L'a');
			}
		}
		return result;
	}

	bool EqualsPath(const std::wstring& left, const std::wstring& right)
	{
		return CompareStringOrdinal(left.c_str(), -1, right.c_str(), -1, TRUE) == CSTR_EQUAL;
	}

	bool StartsWithPath(const std::wstring& path, const std::wstring& root)
	{
		if (path.size() <= root.size())
		{
			return false;
		}
		if (CompareStringOrdinal(path.c_str(), (int)root.size(), root.c_str(), (int)root.size(), TRUE) != CSTR_EQUAL)
		{
			return false;
		}
		return path[root.size()] == L'\\';
	}

	std::wstring JoinPath(const std::wstring& left, const std::wstring& right)
	{
		if (left.empty())
		{
			return right;
		}
		if (right.empty())
		{
			return left;
		}
		if (left.back() == L'\\')
		{
			return left + right;
		}
		return left + L"\\" + right;
	}

	std::wstring RemoveLongPathPrefix(std::wstring path)
	{
		if (path.rfind(L"\\\\?\\UNC\\", 0) == 0)
		{
			return L"\\\\" + path.substr(8);
		}
		if (path.rfind(L"\\\\?\\", 0) == 0)
		{
			return path.substr(4);
		}
		return path;
	}

	bool GetFinalPath(const std::wstring& path, std::wstring& finalPath)
	{
		DWORD attributes = GetFileAttributesW(path.c_str());
		DWORD flags = (attributes != INVALID_FILE_ATTRIBUTES && (attributes & FILE_ATTRIBUTE_DIRECTORY) != 0) ? FILE_FLAG_BACKUP_SEMANTICS : 0;
		HANDLE file = CreateFileW(path.c_str(), FILE_READ_ATTRIBUTES, FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE, nullptr, OPEN_EXISTING, flags, nullptr);
		if (file == INVALID_HANDLE_VALUE)
		{
			return false;
		}
		wchar_t buffer[32768]{};
		DWORD length = GetFinalPathNameByHandleW(file, buffer, (DWORD)std::size(buffer), 0);
		CloseHandle(file);
		if (length == 0 || length >= std::size(buffer))
		{
			return false;
		}
		finalPath = RemoveLongPathPrefix(std::wstring(buffer, length));
		return !finalPath.empty();
	}

	std::wstring ParentPath(const std::wstring& path)
	{
		size_t position = path.find_last_of(L"\\/");
		if (position == std::wstring::npos)
		{
			return L"";
		}
		if (position == 2 && path.size() >= 3 && path[1] == L':')
		{
			return path.substr(0, 3);
		}
		return path.substr(0, position);
	}

	bool FileExists(const std::wstring& path)
	{
		DWORD attributes = GetFileAttributesW(path.c_str());
		return attributes != INVALID_FILE_ATTRIBUTES && (attributes & FILE_ATTRIBUTE_DIRECTORY) == 0;
	}

	bool DirectoryExists(const std::wstring& path)
	{
		DWORD attributes = GetFileAttributesW(path.c_str());
		return attributes != INVALID_FILE_ATTRIBUTES && (attributes & FILE_ATTRIBUTE_DIRECTORY) != 0;
	}

	bool EnsureDirectory(const std::wstring& path)
	{
		if (path.empty())
		{
			return false;
		}
		if (DirectoryExists(path))
		{
			return true;
		}
		std::wstring parent = ParentPath(path);
		if (!parent.empty() && !EqualsPath(parent, path))
		{
			if (!EnsureDirectory(parent))
			{
				return false;
			}
		}
		if (CreateDirectoryW(path.c_str(), nullptr))
		{
			return true;
		}
		DWORD error = GetLastError();
		return error == ERROR_ALREADY_EXISTS && DirectoryExists(path);
	}

	bool IsReservedDosName(const std::wstring& component)
	{
		std::wstring base = component.substr(0, component.find(L'.'));
		base = ToLowerAscii(base);
		if (base == L"con" || base == L"prn" || base == L"aux" || base == L"nul" || base == L"clock$" || base == L"conin$" || base == L"conout$")
		{
			return true;
		}
		if (base.size() == 4 && (base.rfind(L"com", 0) == 0 || base.rfind(L"lpt", 0) == 0))
		{
			return base[3] >= L'1' && base[3] <= L'9';
		}
		return false;
	}

	bool IsSafeRelativePath(const std::wstring& path);
	std::wstring NormalizeRelativePath(const std::wstring& path);

	bool IsReparsePoint(const std::wstring& path)
	{
		DWORD attributes = GetFileAttributesW(path.c_str());
		return attributes != INVALID_FILE_ATTRIBUTES && (attributes & FILE_ATTRIBUTE_REPARSE_POINT) != 0;
	}

	bool IsUpdaterConfigPath(const std::wstring& relativePath)
	{
		return EqualsPath(NormalizeRelativePath(relativePath), kUpdaterConfigName);
	}

	//Creates and validates the destination parent without following a junction/symlink that a
	//previous local user could have placed inside the target App tree. The resolved App root
	//itself is allowed to be redirected by a legitimate system LocalAppData redirection.
	bool PrepareSafeDestination(const std::wstring& root, const std::wstring& relativePath, std::wstring& destination)
	{
		if (!IsSafeRelativePath(relativePath) || !DirectoryExists(root))
		{
			return false;
		}
		std::wstring finalRoot;
		if (!GetFinalPath(root, finalRoot))
		{
			return false;
		}
		destination = JoinPath(root, relativePath);
		if (!StartsWithPath(destination, root))
		{
			return false;
		}

		std::wstring current = root;
		size_t start = 0;
		while (true)
		{
			size_t end = relativePath.find_first_of(L"\\/", start);
			if (end == std::wstring::npos)
			{
				break;
			}
			std::wstring component = relativePath.substr(start, end - start);
			if (component.empty() || component == L"." || component == L"..")
			{
				return false;
			}
			current = JoinPath(current, component);
			if (DirectoryExists(current))
			{
				if (IsReparsePoint(current))
				{
					return false;
				}
			}
			else if (!EnsureDirectory(current) || IsReparsePoint(current))
			{
				return false;
			}
			std::wstring finalCurrent;
			if (!GetFinalPath(current, finalCurrent) || !(EqualsPath(finalCurrent, finalRoot) || StartsWithPath(finalCurrent, finalRoot)))
			{
				return false;
			}
			start = end + 1;
		}

		if (FileExists(destination) && IsReparsePoint(destination))
		{
			return false;
		}
		return true;
	}

	bool IsSafeRelativePath(const std::wstring& path)
	{
		if (path.empty() || path.size() > 1024)
		{
			return false;
		}
		if (path[0] == L'\\' || path[0] == L'/' || path.find(L':') != std::wstring::npos || path.back() == L'\\' || path.back() == L'/')
		{
			return false;
		}
		if (path.find(L"//") != std::wstring::npos || path.find(L"\\\\") != std::wstring::npos)
		{
			return false;
		}
		size_t start = 0;
		while (start <= path.size())
		{
			size_t end = path.find_first_of(L"\\/", start);
			std::wstring component = path.substr(start, end == std::wstring::npos ? std::wstring::npos : end - start);
			if (component.empty() || component == L"." || component == L".." || component.size() > 255)
			{
				return false;
			}
			//Windows silently trims trailing dots/spaces, so these are aliases of a different entry.
			if (component.back() == L'.' || component.back() == L' ')
			{
				return false;
			}
			if (IsReservedDosName(component))
			{
				return false;
			}
			for (wchar_t character : component)
			{
				if (character < 0x20 || character > 0x7e || character == L'<' || character == L'>' || character == L':' || character == L'"' || character == L'|' || character == L'?' || character == L'*')
				{
					return false;
				}
			}
			if (end == std::wstring::npos)
			{
				break;
			}
			start = end + 1;
		}
		return true;
	}

	std::wstring NormalizeRelativePath(const std::wstring& path)
	{
		std::wstring result = path;
		std::replace(result.begin(), result.end(), L'/', L'\\');
		return result;
	}

	bool RemoveDirectoryTree(const std::wstring& path, bool removeRoot)
	{
		if (!DirectoryExists(path))
		{
			return true;
		}
		std::wstring pattern = JoinPath(path, L"*");
		WIN32_FIND_DATAW data{};
		HANDLE findHandle = FindFirstFileW(pattern.c_str(), &data);
		if (findHandle != INVALID_HANDLE_VALUE)
		{
			do
			{
				if (wcscmp(data.cFileName, L".") == 0 || wcscmp(data.cFileName, L"..") == 0)
				{
					continue;
				}
				std::wstring child = JoinPath(path, data.cFileName);
				if ((data.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0)
				{
					RemoveDirectoryTree(child, true);
				}
				else
				{
					SetFileAttributesW(child.c_str(), FILE_ATTRIBUTE_NORMAL);
					DeleteFileW(child.c_str());
				}
			} while (FindNextFileW(findHandle, &data));
			FindClose(findHandle);
		}
		if (removeRoot)
		{
			return RemoveDirectoryW(path.c_str()) != FALSE;
		}
		return true;
	}

	std::wstring ErrorText(DWORD error)
	{
		wchar_t* buffer = nullptr;
		DWORD length = FormatMessageW(FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS,
			nullptr, error, 0, (LPWSTR)&buffer, 0, nullptr);
		std::wstring message = length && buffer ? std::wstring(buffer, length) : L"未知错误";
		if (buffer)
		{
			LocalFree(buffer);
		}
		message = Trim(message);
		return message;
	}

	// -------------------------------------------------------------------------
	// Files, hashes and JSON reports
	// -------------------------------------------------------------------------

	bool ReadFileBytes(const std::wstring& path, std::vector<BYTE>& bytes, size_t maximumSize)
	{
		bytes.clear();
		HANDLE file = CreateFileW(path.c_str(), GENERIC_READ, FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE, nullptr, OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, nullptr);
		if (file == INVALID_HANDLE_VALUE)
		{
			return false;
		}
		LARGE_INTEGER size{};
		if (!GetFileSizeEx(file, &size) || size.QuadPart < 0 || (unsigned long long)size.QuadPart > maximumSize)
		{
			CloseHandle(file);
			return false;
		}
		bytes.resize((size_t)size.QuadPart);
		size_t offset = 0;
		while (offset < bytes.size())
		{
			DWORD read = 0;
			DWORD request = (DWORD)std::min<size_t>(bytes.size() - offset, 1024 * 1024);
			if (!ReadFile(file, bytes.data() + offset, request, &read, nullptr) || read == 0)
			{
				CloseHandle(file);
				bytes.clear();
				return false;
			}
			offset += read;
		}
		CloseHandle(file);
		return true;
	}

	bool WriteFileBytes(const std::wstring& path, const BYTE* bytes, size_t size)
	{
		HANDLE file = CreateFileW(path.c_str(), GENERIC_WRITE, 0, nullptr, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, nullptr);
		if (file == INVALID_HANDLE_VALUE)
		{
			return false;
		}
		size_t offset = 0;
		bool ok = true;
		while (offset < size)
		{
			DWORD written = 0;
			DWORD request = (DWORD)std::min<size_t>(size - offset, 1024 * 1024);
			if (!WriteFile(file, bytes + offset, request, &written, nullptr) || written == 0)
			{
				ok = false;
				break;
			}
			offset += written;
		}
		FlushFileBuffers(file);
		CloseHandle(file);
		if (!ok)
		{
			DeleteFileW(path.c_str());
		}
		return ok;
	}

	bool WriteFileText(const std::wstring& path, const std::string& text, bool atomic)
	{
		std::wstring target = path;
		if (atomic)
		{
			target = path + L".tmp-" + std::to_wstring(GetCurrentProcessId());
		}
		if (!WriteFileBytes(target, reinterpret_cast<const BYTE*>(text.data()), text.size()))
		{
			return false;
		}
		if (atomic)
		{
			if (!MoveFileExW(target.c_str(), path.c_str(), MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH))
			{
				DeleteFileW(target.c_str());
				return false;
			}
		}
		return true;
	}

	std::string WideToUtf8(const std::wstring& value)
	{
		if (value.empty())
		{
			return std::string();
		}
		int length = WideCharToMultiByte(CP_UTF8, 0, value.data(), (int)value.size(), nullptr, 0, nullptr, nullptr);
		if (length <= 0)
		{
			return std::string();
		}
		std::string result(length, '\0');
		WideCharToMultiByte(CP_UTF8, 0, value.data(), (int)value.size(), result.data(), length, nullptr, nullptr);
		return result;
	}

	std::string JsonString(const std::wstring& value)
	{
		std::string utf8 = WideToUtf8(value);
		std::string result = "\"";
		for (unsigned char character : utf8)
		{
			switch (character)
			{
			case '"': result += "\\\""; break;
			case '\\': result += "\\\\"; break;
			case '\b': result += "\\b"; break;
			case '\f': result += "\\f"; break;
			case '\n': result += "\\n"; break;
			case '\r': result += "\\r"; break;
			case '\t': result += "\\t"; break;
			default:
				if (character < 0x20)
				{
					char buffer[8]{};
					sprintf_s(buffer, sizeof(buffer), "\\u%04x", character);
					result += buffer;
				}
				else
				{
					result += (char)character;
				}
				break;
			}
		}
		result += "\"";
		return result;
	}

	std::string JsonBool(bool value)
	{
		return value ? "true" : "false";
	}

	std::string JsonNumber(unsigned long long value)
	{
		return std::to_string(value);
	}

	bool BCryptSha256(const BYTE* data, size_t size, BYTE digest[32])
	{
		BCRYPT_ALG_HANDLE algorithm = nullptr;
		BCRYPT_HASH_HANDLE hash = nullptr;
		bool ok = false;
		if (BCryptOpenAlgorithmProvider(&algorithm, BCRYPT_SHA256_ALGORITHM, nullptr, 0) >= 0)
		{
			if (BCryptCreateHash(algorithm, &hash, nullptr, 0, nullptr, 0, 0) >= 0)
			{
				size_t offset = 0;
				while (offset < size)
				{
					ULONG chunk = (ULONG)std::min<size_t>(size - offset, 1024 * 1024);
					if (BCryptHashData(hash, const_cast<BYTE*>(data + offset), chunk, 0) < 0)
					{
						break;
					}
					offset += chunk;
				}
				if (offset == size && BCryptFinishHash(hash, digest, 32, 0) >= 0)
				{
					ok = true;
				}
			}
		}
		if (hash)
		{
			BCryptDestroyHash(hash);
		}
		if (algorithm)
		{
			BCryptCloseAlgorithmProvider(algorithm, 0);
		}
		return ok;
	}

	bool Sha256File(const std::wstring& path, BYTE digest[32], unsigned long long* sizeOut)
	{
		HANDLE file = CreateFileW(path.c_str(), GENERIC_READ, FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE, nullptr, OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, nullptr);
		if (file == INVALID_HANDLE_VALUE)
		{
			return false;
		}
		LARGE_INTEGER size{};
		if (!GetFileSizeEx(file, &size) || size.QuadPart < 0)
		{
			CloseHandle(file);
			return false;
		}
		if (sizeOut)
		{
			*sizeOut = (unsigned long long)size.QuadPart;
		}
		BCRYPT_ALG_HANDLE algorithm = nullptr;
		BCRYPT_HASH_HANDLE hash = nullptr;
		bool ok = false;
		if (BCryptOpenAlgorithmProvider(&algorithm, BCRYPT_SHA256_ALGORITHM, nullptr, 0) >= 0 &&
			BCryptCreateHash(algorithm, &hash, nullptr, 0, nullptr, 0, 0) >= 0)
		{
			std::vector<BYTE> buffer(1024 * 1024);
			unsigned long long offset = 0;
			while (offset < (unsigned long long)size.QuadPart)
			{
				DWORD request = (DWORD)std::min<unsigned long long>(buffer.size(), (unsigned long long)size.QuadPart - offset);
				DWORD read = 0;
				if (!ReadFile(file, buffer.data(), request, &read, nullptr) || read == 0)
				{
					break;
				}
				if (BCryptHashData(hash, buffer.data(), read, 0) < 0)
				{
					break;
				}
				offset += read;
			}
			if (offset == (unsigned long long)size.QuadPart && BCryptFinishHash(hash, digest, 32, 0) >= 0)
			{
				ok = true;
			}
		}
		if (hash)
		{
			BCryptDestroyHash(hash);
		}
		if (algorithm)
		{
			BCryptCloseAlgorithmProvider(algorithm, 0);
		}
		CloseHandle(file);
		return ok;
	}

	std::wstring ToHex(const BYTE* bytes, size_t size)
	{
		static const wchar_t hex[] = L"0123456789abcdef";
		std::wstring result;
		result.reserve(size * 2);
		for (size_t index = 0; index < size; index++)
		{
			result.push_back(hex[bytes[index] >> 4]);
			result.push_back(hex[bytes[index] & 0x0f]);
		}
		return result;
	}

	bool IsSha256String(const std::wstring& value)
	{
		if (value.size() != 64)
		{
			return false;
		}
		for (wchar_t character : value)
		{
			if (!((character >= L'0' && character <= L'9') || (character >= L'a' && character <= L'f') || (character >= L'A' && character <= L'F')))
			{
				return false;
			}
		}
		return true;
	}

	// -------------------------------------------------------------------------
	// Embedded payload and FDI extraction
	// -------------------------------------------------------------------------

	bool LoadEmbeddedPayload(HINSTANCE instance)
	{
		HRSRC resource = FindResourceW(instance, MAKEINTRESOURCEW(kPayloadResourceId), RT_RCDATA);
		if (!resource)
		{
			return false;
		}
		DWORD size = SizeofResource(instance, resource);
		HGLOBAL loaded = LoadResource(instance, resource);
		if (!loaded || size == 0)
		{
			return false;
		}
		const BYTE* data = static_cast<const BYTE*>(LockResource(loaded));
		if (!data)
		{
			return false;
		}
		g_embeddedCab.data = data;
		g_embeddedCab.size = size;
		return true;
	}

	void* DIAMONDAPI FdiAlloc(ULONG size)
	{
		return HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, size);
	}

	void DIAMONDAPI FdiFree(void* pointer)
	{
		if (pointer)
		{
			HeapFree(GetProcessHeap(), 0, pointer);
		}
	}

	INT_PTR DIAMONDAPI FdiOpen(LPSTR fileName, int oflag, int)
	{
		AppendFdiTrace("open " + std::string(fileName ? fileName : "(null)") + " flags=" + std::to_string(oflag));
		// The only file FDI needs to read is the embedded cabinet. The cabinet name is synthetic.
		if (fileName && strcmp(fileName, "payload.cab") != 0)
		{
			g_extractError = "FDI requested an unexpected cabinet path.";
			return -1;
		}
		CabReadHandle* reader = new (std::nothrow) CabReadHandle();
		if (!reader)
		{
			g_extractError = "Could not allocate a cabinet reader.";
			return -1;
		}
		g_cabReadHandles.insert((INT_PTR)reader);
		return (INT_PTR)reader;
	}

	UINT DIAMONDAPI FdiRead(INT_PTR handle, void* buffer, UINT bytes)
	{
		auto readerEntry = g_cabReadHandles.find(handle);
		if (readerEntry == g_cabReadHandles.end())
		{
			DWORD read = 0;
			return ReadFile((HANDLE)handle, buffer, bytes, &read, nullptr) ? read : 0;
		}
		CabReadHandle* reader = (CabReadHandle*)handle;
		if (reader->position >= g_embeddedCab.size)
		{
			AppendFdiTrace("read eof pos=" + std::to_string(reader->position));
			return 0;
		}
		DWORD available = g_embeddedCab.size - reader->position;
		DWORD read = std::min<DWORD>(available, bytes);
		memcpy(buffer, g_embeddedCab.data + reader->position, read);
		reader->position += read;
		AppendFdiTrace("read bytes=" + std::to_string((unsigned long long)read) + " pos=" + std::to_string(reader->position));
		return read;
	}

	UINT DIAMONDAPI FdiWrite(INT_PTR handle, void* buffer, UINT bytes)
	{
		if (g_cabReadHandles.find(handle) != g_cabReadHandles.end())
		{
			return 0;
		}
		DWORD written = 0;
		UINT result = WriteFile((HANDLE)handle, buffer, bytes, &written, nullptr) ? written : 0;
		if (result != bytes)
		{
			AppendFdiTrace("write failed bytes=" + std::to_string(bytes) + " wrote=" + std::to_string(result));
		}
		return result;
	}

	long DIAMONDAPI FdiSeek(INT_PTR handle, long distance, int type)
	{
		auto readerEntry = g_cabReadHandles.find(handle);
		if (readerEntry == g_cabReadHandles.end())
		{
			LARGE_INTEGER move{};
			move.QuadPart = distance;
			LARGE_INTEGER result{};
			return SetFilePointerEx((HANDLE)handle, move, &result, type) ? (int)result.QuadPart : -1;
		}
		CabReadHandle* reader = (CabReadHandle*)handle;
		long long position = reader->position;
		long long target = 0;
		if (type == FILE_BEGIN)
		{
			target = distance;
		}
		else if (type == FILE_CURRENT)
		{
			target = position + distance;
		}
		else if (type == FILE_END)
		{
			target = (long long)g_embeddedCab.size + distance;
		}
		else
		{
			return -1;
		}
		if (target < 0 || (unsigned long long)target > g_embeddedCab.size)
		{
			AppendFdiTrace("seek invalid type=" + std::to_string(type) + " distance=" + std::to_string(distance) + " target=" + std::to_string(target));
			return -1;
		}
		reader->position = (DWORD)target;
		AppendFdiTrace("seek type=" + std::to_string(type) + " distance=" + std::to_string(distance) + " target=" + std::to_string(target));
		return (long)target;
	}

	int DIAMONDAPI FdiClose(INT_PTR handle)
	{
		AppendFdiTrace("close hf=" + std::to_string((long long)handle));
		auto readerEntry = g_cabReadHandles.find(handle);
		if (readerEntry != g_cabReadHandles.end())
		{
			g_cabReadHandles.erase(readerEntry);
			delete (CabReadHandle*)handle;
			return 0;
		}
		return CloseHandle((HANDLE)handle) ? 0 : -1;
	}

	std::wstring ConvertCabName(const char* name)
	{
		if (!name)
		{
			return L"";
		}
		std::wstring result;
		for (const char* current = name; *current; current++)
		{
			unsigned char character = (unsigned char)*current;
			if (character > 0x7e)
			{
				return L"";
			}
			result.push_back((wchar_t)character);
		}
		return NormalizeRelativePath(result);
	}

	INT_PTR DIAMONDAPI FdiNotify(FDINOTIFICATIONTYPE notification, PFDINOTIFICATION info)
	{
		if (!info)
		{
			return -1;
		}
		if (notification == fdintCABINET_INFO)
		{
			return 0;
		}
		if (notification == fdintNEXT_CABINET)
		{
			g_extractError = "Unexpected multi-cabinet payload.";
			return -1;
		}
		if (notification == fdintCOPY_FILE)
		{
			std::wstring relative = ConvertCabName(info->psz1);
			AppendFdiTrace("copy " + WideToUtf8(relative) + " size=" + std::to_string(info->cb));
			if (!IsSafeRelativePath(relative))
			{
				g_extractError = "Cabinet entry path is not a safe relative path.";
				return -1;
			}
			std::wstring lower = ToLowerAscii(relative);
			if (g_extractedNames.find(lower) != g_extractedNames.end())
			{
				g_extractError = "Duplicate cabinet entry.";
				return -1;
			}
			std::wstring destination;
			if (!PrepareSafeDestination(g_stageRoot, relative, destination))
			{
				g_extractError = "Cabinet entry escapes the staging root or traverses a reparse point.";
				return -1;
			}
			HANDLE file = CreateFileW(destination.c_str(), GENERIC_WRITE, 0, nullptr, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL | FILE_FLAG_SEQUENTIAL_SCAN, nullptr);
			if (file == INVALID_HANDLE_VALUE)
			{
				g_extractError = "Could not create staging file: " + WideToUtf8(relative);
				return -1;
			}
			g_extractedNames.insert(lower);
			return (INT_PTR)file;
		}
		if (notification == fdintCLOSE_FILE_INFO)
		{
			AppendFdiTrace("close-file-info");
			HANDLE file = (HANDLE)info->hf;
			if (info->date || info->time)
			{
				FILETIME fileTime{};
				if (DosDateTimeToFileTime(info->date, info->time, &fileTime))
				{
					SetFileTime(file, &fileTime, &fileTime, &fileTime);
				}
			}
			return CloseHandle(file) ? TRUE : FALSE;
		}
		return 0;
	}

	bool ExtractEmbeddedPayload(const std::wstring& stageRoot)
	{
		g_stageRoot = stageRoot;
		g_extractedNames.clear();
		g_extractError.clear();
		g_fdiTrace.clear();
		g_cabReadHandles.clear();

		ERF error{};
		HFDI fdi = FDICreate(FdiAlloc, FdiFree, FdiOpen, FdiRead, FdiWrite, FdiClose, FdiSeek, cpuUNKNOWN, &error);
		if (!fdi)
		{
			g_extractError = "FDICreate failed.";
			return false;
		}
		BOOL copied = FDICopy(fdi, const_cast<char*>("payload.cab"), const_cast<char*>(""), 0, FdiNotify, nullptr, nullptr);
		DWORD fdiError = error.erfOper;
		FDIDestroy(fdi);
		if (!copied || !g_extractError.empty())
		{
			if (g_extractError.empty())
			{
				g_extractError = "FDICopy failed with error " + std::to_string(fdiError) + ".";
			}
			return false;
		}
		return true;
	}

	// -------------------------------------------------------------------------
	// Manifest and state files
	// -------------------------------------------------------------------------

	bool VerifyManifestId(const std::wstring& root)
	{
		std::vector<BYTE> bytes;
		if (!ReadFileBytes(JoinPath(root, kManifestName), bytes, kMaxManifestBytes))
		{
			return false;
		}
		BYTE digest[32]{};
		return BCryptSha256(bytes.data(), bytes.size(), digest) && ToHex(digest, 32) == RT_PAYLOAD_ID;
	}

	bool LoadManifest(const std::wstring& root, PayloadManifest& manifest)
	{
		std::vector<BYTE> bytes;
		if (!ReadFileBytes(JoinPath(root, kManifestName), bytes, kMaxManifestBytes))
		{
			return false;
		}
		std::string text(reinterpret_cast<const char*>(bytes.data()), bytes.size());
		if (text.size() >= 3 && (unsigned char)text[0] == 0xef && (unsigned char)text[1] == 0xbb && (unsigned char)text[2] == 0xbf)
		{
			text.erase(0, 3);
		}
		size_t lineStart = 0;
		bool firstLine = true;
		std::set<std::wstring> seen;
		manifest.files.clear();
		manifest.coreSha256.clear();
		while (lineStart <= text.size())
		{
			size_t lineEnd = text.find('\n', lineStart);
			std::string line = text.substr(lineStart, lineEnd == std::string::npos ? std::string::npos : lineEnd - lineStart);
			if (!line.empty() && line.back() == '\r')
			{
				line.pop_back();
			}
			if (firstLine)
			{
				if (line != "RTPAYLOAD 1")
				{
					return false;
				}
				firstLine = false;
			}
			else if (!line.empty())
			{
				size_t first = line.find('\t');
				size_t second = first == std::string::npos ? std::string::npos : line.find('\t', first + 1);
				if (first == std::string::npos || second == std::string::npos)
				{
					return false;
				}
				std::string hashText = line.substr(0, first);
				std::string sizeText = line.substr(first + 1, second - first - 1);
				std::string pathText = line.substr(second + 1);
				if (hashText.size() != 64 || pathText.empty() || pathText.find('\t') != std::string::npos)
				{
					return false;
				}
				std::wstring relative;
				for (unsigned char character : pathText)
				{
					if (character > 0x7e)
					{
						return false;
					}
					relative.push_back((wchar_t)character);
				}
				relative = NormalizeRelativePath(relative);
				if (!IsSafeRelativePath(relative))
				{
					return false;
				}
				std::wstring lower = ToLowerAscii(relative);
				if (!seen.insert(lower).second)
				{
					return false;
				}
				unsigned long long size = 0;
				try
				{
					size = std::stoull(sizeText);
				}
				catch (...)
				{
					return false;
				}
				std::wstring hash;
				for (char character : hashText)
				{
					if (!((character >= '0' && character <= '9') || (character >= 'a' && character <= 'f') || (character >= 'A' && character <= 'F')))
					{
						return false;
					}
					hash.push_back((wchar_t)character);
				}
				PayloadFile file;
				file.relativePath = relative;
				file.sha256 = ToLowerAscii(hash);
				file.size = size;
				manifest.files.push_back(file);
				if (EqualsPath(relative, kCoreName))
				{
					manifest.coreSha256 = file.sha256;
				}
			}
			if (lineEnd == std::string::npos)
			{
				break;
			}
			lineStart = lineEnd + 1;
		}
		return !manifest.files.empty() && !manifest.coreSha256.empty();
	}

	bool VerifyFiles(const std::wstring& root, const PayloadManifest& manifest, bool requireNoExtras, bool skipUpdaterConfig = false)
	{
		std::map<std::wstring, const PayloadFile*> expected;
		for (const PayloadFile& file : manifest.files)
		{
			expected[ToLowerAscii(file.relativePath)] = &file;
		}
		for (const PayloadFile& file : manifest.files)
		{
			//Settings\Updater.json is a seed for first run and belongs to the user afterwards.
			if (skipUpdaterConfig && IsUpdaterConfigPath(file.relativePath))
			{
				continue;
			}
			std::wstring path = JoinPath(root, file.relativePath);
			unsigned long long actualSize = 0;
			BYTE digest[32]{};
			if (!Sha256File(path, digest, &actualSize) || actualSize != file.size || ToHex(digest, 32) != file.sha256)
			{
				return false;
			}
		}
		if (!requireNoExtras)
		{
			// The App directory may contain user Settings, Profiles and other files. Only the
			// manifest-listed program files are validated there.
			return true;
		}

		std::vector<std::wstring> pending;
		pending.push_back(root);
		size_t seenFiles = 0;
		while (!pending.empty())
		{
			std::wstring directory = pending.back();
			pending.pop_back();
			WIN32_FIND_DATAW data{};
			HANDLE findHandle = FindFirstFileW(JoinPath(directory, L"*").c_str(), &data);
			if (findHandle == INVALID_HANDLE_VALUE)
			{
				return false;
			}
			do
			{
				if (wcscmp(data.cFileName, L".") == 0 || wcscmp(data.cFileName, L"..") == 0)
				{
					continue;
				}
				std::wstring path = JoinPath(directory, data.cFileName);
				if ((data.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0)
				{
					if ((data.dwFileAttributes & FILE_ATTRIBUTE_REPARSE_POINT) != 0)
					{
						FindClose(findHandle);
						return false;
					}
					pending.push_back(path);
					continue;
				}
				std::wstring relative = path.substr(root.size() + 1);
				if (EqualsPath(relative, kManifestName))
				{
					continue;
				}
				if ((data.dwFileAttributes & FILE_ATTRIBUTE_REPARSE_POINT) != 0)
				{
					FindClose(findHandle);
					return false;
				}
				auto found = expected.find(ToLowerAscii(relative));
				if (found == expected.end())
				{
					FindClose(findHandle);
					return false;
				}
				seenFiles++;
				expected.erase(found);
			} while (FindNextFileW(findHandle, &data));
			FindClose(findHandle);
		}
		return expected.empty() && seenFiles == manifest.files.size();
	}

	PortableState LoadState(const std::wstring& appDirectory)
	{
		PortableState state;
		std::vector<BYTE> bytes;
		if (!ReadFileBytes(JoinPath(appDirectory, kStateName), bytes, kMaxStateBytes))
		{
			return state;
		}
		std::string text(reinterpret_cast<const char*>(bytes.data()), bytes.size());
		size_t lineStart = 0;
		bool firstLine = true;
		bool havePayload = false;
		bool haveCore = false;
		while (lineStart <= text.size())
		{
			size_t lineEnd = text.find('\n', lineStart);
			std::string line = text.substr(lineStart, lineEnd == std::string::npos ? std::string::npos : lineEnd - lineStart);
			if (!line.empty() && line.back() == '\r')
			{
				line.pop_back();
			}
			if (firstLine)
			{
				if (line != "RTPORTABLE 1")
				{
					return PortableState{};
				}
				firstLine = false;
			}
			else if (!line.empty())
			{
				size_t space = line.find(' ');
				if (space == std::string::npos)
				{
					return PortableState{};
				}
				std::string key = line.substr(0, space);
				std::string value = line.substr(space + 1);
				std::wstring wideValue(value.begin(), value.end());
				if (key == "payload" && IsSha256String(wideValue))
				{
					state.payload = ToLowerAscii(wideValue);
					havePayload = true;
				}
				else if (key == "core" && IsSha256String(wideValue))
				{
					state.coreSha256 = ToLowerAscii(wideValue);
					haveCore = true;
				}
				else if (key == "external")
				{
					state.external = value == "1";
				}
			}
			if (lineEnd == std::string::npos)
			{
				break;
			}
			lineStart = lineEnd + 1;
		}
		state.valid = firstLine == false && havePayload && haveCore;
		return state;
	}

	std::wstring StateText(const std::wstring& payload, const std::wstring& coreSha256, bool external)
	{
		return kStateHeader + std::wstring(L"\npayload ") + payload + L"\ncore " + coreSha256 + L"\nexternal " + (external ? L"1" : L"0") + L"\n";
	}

	bool SaveState(const std::wstring& appDirectory, const std::wstring& payload, const std::wstring& coreSha256, bool external)
	{
		std::wstring text = StateText(payload, coreSha256, external);
		return WriteFileText(JoinPath(appDirectory, kStateName), WideToUtf8(text), true);
	}

	bool SavePending(const std::wstring& appDirectory, const std::wstring& payload)
	{
		std::wstring text = kStateHeader + std::wstring(L"\npending ") + payload + L"\n";
		return WriteFileText(JoinPath(appDirectory, kPendingName), WideToUtf8(text), false);
	}

	// -------------------------------------------------------------------------
	// Application paths, running-core detection and launch
	// -------------------------------------------------------------------------

	bool GetAppPaths(std::wstring& rootDirectory, std::wstring& appDirectory)
	{
		PWSTR localAppData = nullptr;
		if (FAILED(SHGetKnownFolderPath(FOLDERID_LocalAppData, KF_FLAG_CREATE, nullptr, &localAppData)) || !localAppData)
		{
			return false;
		}
		rootDirectory = JoinPath(localAppData, RT_APP_FOLDER);
		CoTaskMemFree(localAppData);
		appDirectory = JoinPath(rootDirectory, L"App");
		return !rootDirectory.empty() && !appDirectory.empty();
	}

	bool ProcessImageMatches(const std::wstring& expectedPath)
	{
		HANDLE snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
		if (snapshot == INVALID_HANDLE_VALUE)
		{
			return false;
		}
		PROCESSENTRY32W entry{};
		entry.dwSize = sizeof(entry);
		bool matched = false;
		if (Process32FirstW(snapshot, &entry))
		{
			do
			{
				if (_wcsicmp(entry.szExeFile, kCoreName) != 0)
				{
					continue;
				}
				HANDLE process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, FALSE, entry.th32ProcessID);
				if (!process)
				{
					continue;
				}
				wchar_t processPath[32768]{};
				DWORD processPathSize = (DWORD)std::size(processPath);
				if (QueryFullProcessImageNameW(process, 0, processPath, &processPathSize) && EqualsPath(processPath, expectedPath))
				{
					matched = true;
				}
				CloseHandle(process);
				if (matched)
				{
					break;
				}
			} while (Process32NextW(snapshot, &entry));
		}
		CloseHandle(snapshot);
		return matched;
	}

	bool IsCoreRunning(const std::wstring& corePath)
	{
		if (!FileExists(corePath))
		{
			return false;
		}
		std::wstring expectedPath = corePath;
		std::wstring resolvedPath;
		if (GetFinalPath(corePath, resolvedPath))
		{
			expectedPath = resolvedPath;
		}
		if (ProcessImageMatches(expectedPath))
		{
			return true;
		}
		HANDLE file = CreateFileW(corePath.c_str(), GENERIC_READ | GENERIC_WRITE, 0, nullptr, OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, nullptr);
		if (file != INVALID_HANDLE_VALUE)
		{
			CloseHandle(file);
			return false;
		}
		return GetLastError() == ERROR_SHARING_VIOLATION;
	}

	struct AppDirectoryLock
	{
		HANDLE handle = nullptr;

		~AppDirectoryLock()
		{
			if (handle)
			{
				CloseHandle(handle);
			}
		}
	};

	bool AcquireAppDirectoryLock(const std::wstring& rootDirectory, const std::wstring&, AppDirectoryLock& lock, std::wstring& error)
	{
		//Both launchers must derive the same name even when App does not exist yet. Create the
		//root, resolve that root through a legitimate LocalAppData redirection, and always use
		//resolved-root\App. Resolving App only when it happens to exist would let the first
		//launcher compute a different name while it creates that directory.
		if (!DirectoryExists(rootDirectory) && !EnsureDirectory(rootDirectory))
		{
			error = L"无法创建应用数据目录。";
			return false;
		}
		std::wstring resolved;
		if (!GetFinalPath(rootDirectory, resolved))
		{
			error = L"无法解析应用数据目录。";
			return false;
		}
		std::wstring identity = ToLowerAscii(JoinPath(resolved, L"App"));

		BYTE digest[32]{};
		if (!BCryptSha256(reinterpret_cast<const BYTE*>(identity.data()), identity.size() * sizeof(wchar_t), digest))
		{
			error = L"无法生成启动器互斥标识。";
			return false;
		}
		std::wstring mutexName = L"Local\\RadeonTunerZhCN-Portable-" + ToHex(digest, 32);

		HANDLE handle = CreateMutexW(nullptr, FALSE, mutexName.c_str());
		if (!handle)
		{
			error = L"无法创建启动器互斥体：" + ErrorText(GetLastError());
			return false;
		}
		DWORD waitResult = WaitForSingleObject(handle, 120000);
		if (waitResult != WAIT_OBJECT_0 && waitResult != WAIT_ABANDONED)
		{
			DWORD waitError = GetLastError();
			CloseHandle(handle);
			error = waitResult == WAIT_TIMEOUT ? L"另一个启动器仍在准备应用程序，请稍后重试。" : L"等待另一个启动器失败：" + ErrorText(waitError);
			return false;
		}
		lock.handle = handle;
		return true;
	}

	struct RecallWindowSearch
	{
		std::wstring expectedPath;
		bool found = false;
	};

	BOOL CALLBACK RecallWindowCallback(HWND window, LPARAM parameter)
	{
		RecallWindowSearch* search = reinterpret_cast<RecallWindowSearch*>(parameter);
		wchar_t className[128]{};
		if (GetClassNameW(window, className, (int)std::size(className)) == 0 || _wcsicmp(className, kMainWindowClass) != 0)
		{
			return TRUE;
		}
		DWORD processId = 0;
		GetWindowThreadProcessId(window, &processId);
		if (processId == 0)
		{
			return TRUE;
		}
		HANDLE process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, FALSE, processId);
		if (!process)
		{
			return TRUE;
		}
		wchar_t processPath[32768]{};
		DWORD processPathSize = (DWORD)std::size(processPath);
		bool matches = QueryFullProcessImageNameW(process, 0, processPath, &processPathSize) && EqualsPath(processPath, search->expectedPath);
		CloseHandle(process);
		if (!matches)
		{
			return TRUE;
		}
		ShowWindow(window, SW_SHOWNORMAL);
		SetForegroundWindow(window);
		search->found = true;
		return FALSE;
	}

	bool RecallCoreWindow(const std::wstring& corePath)
	{
		RecallWindowSearch search;
		search.expectedPath = corePath;
		std::wstring resolved;
		if (GetFinalPath(corePath, resolved))
		{
			search.expectedPath = resolved;
		}
		EnumWindows(RecallWindowCallback, reinterpret_cast<LPARAM>(&search));
		return search.found;
	}

	bool LaunchCore(const std::wstring& appDirectory, const std::wstring& corePath, const std::wstring& originalCommandLine, DWORD& processId, std::wstring& commandLine)
	{
		if (!FileExists(corePath))
		{
			return false;
		}

		// Resolve filesystem redirections before launching. This keeps the core's own
		// GetModuleFileName path identical to QueryFullProcessImageName for the single-instance
		// matcher, including under packaged test hosts that redirect LOCALAPPDATA.
		std::wstring launchDirectory = appDirectory;
		std::wstring launchCorePath = corePath;
		std::wstring resolvedPath;
		if (GetFinalPath(appDirectory, resolvedPath))
		{
			launchDirectory = resolvedPath;
		}
		if (GetFinalPath(corePath, resolvedPath))
		{
			launchCorePath = resolvedPath;
		}

		commandLine = L"\"" + launchCorePath + L"\"";
		if (!originalCommandLine.empty())
		{
			commandLine += L" " + originalCommandLine;
		}
		STARTUPINFOW startup{};
		startup.cb = sizeof(startup);
		PROCESS_INFORMATION process{};
		if (!SetCurrentDirectoryW(launchDirectory.c_str()))
		{
			return false;
		}
		std::vector<wchar_t> mutableCommandLine(commandLine.begin(), commandLine.end());
		mutableCommandLine.push_back(L'\0');
		BOOL created = CreateProcessW(launchCorePath.c_str(), mutableCommandLine.data(), nullptr, nullptr, FALSE, CREATE_UNICODE_ENVIRONMENT, nullptr, launchDirectory.c_str(), &startup, &process);
		if (!created)
		{
			return false;
		}
		processId = process.dwProcessId;
		CloseHandle(process.hThread);
		CloseHandle(process.hProcess);
		return true;
	}

	// -------------------------------------------------------------------------
	// Optional diagnostics used by the acceptance tests (never by normal users)
	// -------------------------------------------------------------------------

	bool EnvironmentFlag(const wchar_t* name)
	{
		wchar_t value[16]{};
		DWORD length = GetEnvironmentVariableW(name, value, (DWORD)std::size(value));
		return length > 0 && (value[0] == L'1' || value[0] == L't' || value[0] == L'T');
	}

	std::wstring EnvironmentString(const wchar_t* name)
	{
		DWORD length = GetEnvironmentVariableW(name, nullptr, 0);
		if (length == 0)
		{
			return L"";
		}
		std::wstring value(length, L'\0');
		length = GetEnvironmentVariableW(name, value.data(), length);
		if (length == 0)
		{
			return L"";
		}
		value.resize(length);
		return value;
	}

	void WriteReport(const std::wstring& path, const std::string& json)
	{
		if (path.empty())
		{
			return;
		}
		EnsureDirectory(ParentPath(path));
		WriteFileText(path, json + "\n", false);
	}

	std::string BuildReportJson(const std::wstring& mode, const std::wstring& action, bool launched, bool noLaunch, const std::wstring& appDirectory, const std::wstring& corePath, const std::wstring& payloadSha256, const std::wstring& commandLine, DWORD processId, const std::wstring& detail)
	{
		std::string json = "{\n";
		json += "  \"mode\": " + JsonString(mode) + ",\n";
		json += "  \"action\": " + JsonString(action) + ",\n";
		json += "  \"launched\": " + JsonBool(launched) + ",\n";
		json += "  \"no_launch\": " + JsonBool(noLaunch) + ",\n";
		json += "  \"app_directory\": " + JsonString(appDirectory) + ",\n";
		json += "  \"core_path\": " + JsonString(corePath) + ",\n";
		json += "  \"payload_sha256\": " + JsonString(payloadSha256) + ",\n";
		json += "  \"command_line\": " + JsonString(commandLine) + ",\n";
		json += "  \"process_id\": " + std::to_string(processId) + ",\n";
		json += "  \"detail\": " + JsonString(detail) + "\n";
		json += "}\n";
		return json;
	}

	void WriteDiagnosticReport(const std::wstring& mode, const std::wstring& action, bool launched, bool noLaunch, const std::wstring& appDirectory, const std::wstring& corePath, const std::wstring& payloadSha256, const std::wstring& commandLine, DWORD processId, const std::wstring& detail)
	{
		std::wstring path = EnvironmentString(L"RADEONTUNER_LAUNCHER_REPORT");
		if (!path.empty())
		{
			WriteReport(path, BuildReportJson(mode, action, launched, noLaunch, appDirectory, corePath, payloadSha256, commandLine, processId, detail));
		}
	}

	// -------------------------------------------------------------------------
	// Normal startup flow
	// -------------------------------------------------------------------------

	int ShowError(const std::wstring& message, bool suppressUi)
	{
		if (!suppressUi)
		{
			MessageBoxW(nullptr, message.c_str(), L"RadeonTuner", MB_OK | MB_ICONERROR | MB_SETFOREGROUND);
		}
		return 1;
	}

	int RunNormal(HINSTANCE instance, const std::wstring& originalCommandLine)
	{
		const bool suppressUi = EnvironmentFlag(L"RADEONTUNER_LAUNCHER_TEST_NO_UI");
		const bool planOnly = EnvironmentFlag(L"RADEONTUNER_LAUNCHER_TEST_PLAN_ONLY");

		std::wstring rootDirectory;
		std::wstring appDirectory;
		if (!GetAppPaths(rootDirectory, appDirectory))
		{
			return ShowError(L"无法获取当前用户的本地应用数据目录。", suppressUi);
		}

		if (!LoadEmbeddedPayload(instance))
		{
			return ShowError(L"内置应用程序资源损坏或缺失。", suppressUi);
		}
		std::wstring payloadSha256 = RT_PAYLOAD_ID;
		if (!IsSha256String(payloadSha256))
		{
			return ShowError(L"内置应用程序构建指纹无效。", suppressUi);
		}

		//The lock covers the state read, extraction, replacement, completion marker and launch
		//decision. A second launcher waits here instead of interleaving files or clearing a pending
		//journal written by the first launcher.
		AppDirectoryLock appLock;
		std::wstring lockError;
		if (!AcquireAppDirectoryLock(rootDirectory, appDirectory, appLock, lockError))
		{
			return ShowError(L"无法安全启动 RadeonTuner。\n\n" + lockError, suppressUi);
		}

		//Re-read everything after acquiring the lock: a concurrent launcher may already have
		//installed a complete payload and written its completion marker.
		std::wstring corePath = JoinPath(appDirectory, kCoreName);
		std::wstring pendingPath = JoinPath(appDirectory, kPendingName);
		bool pending = FileExists(pendingPath);
		PortableState state = LoadState(appDirectory);

		std::wstring currentCoreSha256;
		if (FileExists(corePath))
		{
			BYTE coreDigest[32]{};
			if (Sha256File(corePath, coreDigest, nullptr))
			{
				currentCoreSha256 = ToHex(coreDigest, 32);
			}
		}

		bool coreRunning = IsCoreRunning(corePath);
		bool coreWindowRecalled = coreRunning && RecallCoreWindow(corePath);

		bool externallyUpdated = false;
		if (!pending && state.valid && !currentCoreSha256.empty() && currentCoreSha256 != state.coreSha256)
		{
			// The core was replaced outside this launcher (normally by RadeonTuner's own updater).
			// Preserve that update instead of overwriting it just because this outer EXE is newer.
			// The pending journal above prevents a half-applied file set from looking external.
			externallyUpdated = true;
			state.external = true;
			state.payload = payloadSha256;
			state.coreSha256 = currentCoreSha256;
			SaveState(appDirectory, state.payload, state.coreSha256, true);
		}

		bool directLaunch = !currentCoreSha256.empty() && !pending &&
			((state.valid && state.external) || (state.valid && state.payload == payloadSha256 && currentCoreSha256 == state.coreSha256));

		if (directLaunch)
		{
			if (planOnly)
			{
				WriteDiagnosticReport(L"normal", L"plan-launch", false, true, appDirectory, corePath, payloadSha256, L"", 0, L"Existing App is current; no extraction needed.");
				return 0;
			}
			if (coreRunning)
			{
				//Never start a second core while the first one is still before its own instance
				//check or has not created RadeonTunerMainWindow yet. Poll for a bounded time so a
				//cold-starting core gets the same recall as an already visible one.
				bool recalled = coreWindowRecalled;
				bool coreAlive = true;
				ULONGLONG waitedMilliseconds = 0;
				if (!recalled)
				{
					const ULONGLONG waitStart = GetTickCount64();
					const ULONGLONG deadline = waitStart + 15000;
					for (;;)
					{
						if (!IsCoreRunning(corePath))
						{
							coreAlive = false;
							break;
						}
						ULONGLONG now = GetTickCount64();
						if (now >= deadline)
						{
							break;
						}
						Sleep((DWORD)std::min<ULONGLONG>(100, deadline - now));
						if (RecallCoreWindow(corePath))
						{
							recalled = true;
							break;
						}
					}
					waitedMilliseconds = GetTickCount64() - waitStart;
				}

				if (recalled)
				{
					WriteDiagnosticReport(L"normal", L"recall-existing", false, true, appDirectory, corePath, payloadSha256, L"", 0,
						L"Recalled RadeonTunerMainWindow of the matching core.");
					return 0;
				}
				if (!coreAlive)
				{
					WriteDiagnosticReport(L"normal", L"recall-core-exited", false, true, appDirectory, corePath, payloadSha256, L"", 0,
						L"Matching core exited before RadeonTunerMainWindow appeared after " + std::to_wstring(waitedMilliseconds) + L" ms; no second core was started.");
					if (!suppressUi)
					{
						MessageBoxW(nullptr,
							L"RadeonTuner 核心进程在显示主窗口前已退出。\n\n未启动第二个实例。请重新启动此程序。",
							L"RadeonTuner", MB_OK | MB_ICONERROR | MB_SETFOREGROUND);
					}
					return 3;
				}
				WriteDiagnosticReport(L"normal", L"recall-window-timeout", false, true, appDirectory, corePath, payloadSha256, L"", 0,
					L"Matching core is still running but RadeonTunerMainWindow did not appear within 15 seconds; no second core was started.");
				if (!suppressUi)
				{
					MessageBoxW(nullptr,
						L"RadeonTuner 核心进程仍在运行，但主窗口在 15 秒内未出现。\n\n未启动第二个实例。请稍后重试。",
						L"RadeonTuner", MB_OK | MB_ICONERROR | MB_SETFOREGROUND);
				}
				return 3;
			}
			DWORD processId = 0;
			std::wstring commandLine;
			if (!LaunchCore(appDirectory, corePath, originalCommandLine, processId, commandLine))
			{
				WriteDiagnosticReport(L"normal", L"launch-failed", false, false, appDirectory, corePath, payloadSha256, commandLine, 0, L"CreateProcess failed: " + ErrorText(GetLastError()));
				return ShowError(L"无法启动 RadeonTuner。\n\n" + ErrorText(GetLastError()), suppressUi);
			}
			WriteDiagnosticReport(L"normal", L"launch-existing", true, false, appDirectory, corePath, payloadSha256, commandLine, processId, externallyUpdated ? L"Preserved a core updated outside this launcher." : L"Payload cache is current.");
			return 0;
		}

		if (planOnly)
		{
			WriteDiagnosticReport(L"normal", L"plan-extract", false, true, appDirectory, corePath, payloadSha256, L"", 0, L"Payload extraction would be required.");
			return 0;
		}

		if (coreRunning)
		{
			WriteDiagnosticReport(L"normal", L"blocked-core-running", false, true, appDirectory, corePath, payloadSha256, L"", 0, L"Core is running; no extraction or forced close was attempted.");
			if (!suppressUi)
			{
				MessageBoxW(nullptr,
					L"RadeonTuner 正在运行，需要更新程序文件。\n\n请先从托盘菜单退出 RadeonTuner，然后重新启动此程序。\n本次启动不会强制关闭正在运行的 RadeonTuner。",
					L"RadeonTuner", MB_OK | MB_ICONINFORMATION | MB_SETFOREGROUND);
			}
			return 4;
		}

		std::wstring stage = JoinPath(rootDirectory, L".portable-stage-" + std::to_wstring(GetCurrentProcessId()) + L"-" + std::to_wstring(GetTickCount64()));
		if (!EnsureDirectory(rootDirectory) || !EnsureDirectory(stage))
		{
			return ShowError(L"无法创建临时释放目录。", suppressUi);
		}

		auto failExtraction = [&](const std::wstring& detail) -> int
		{
			RemoveDirectoryTree(stage, true);
			WriteDiagnosticReport(L"normal", L"extract-failed", false, true, appDirectory, corePath, payloadSha256, L"", 0, detail);
			return ShowError(L"无法完成应用程序释放，未启动 RadeonTuner。\n\n" + detail, suppressUi);
		};

		if (!ExtractEmbeddedPayload(stage))
		{
			std::wstring detail = g_extractError.empty() ? L"未知 CAB 错误" : std::wstring(g_extractError.begin(), g_extractError.end());
			return failExtraction(detail);
		}

		if (!VerifyManifestId(stage))
		{
			return failExtraction(L"释放清单与构建指纹不一致。");
		}
		PayloadManifest manifest;
		if (!LoadManifest(stage, manifest))
		{
			return failExtraction(L"释放清单缺失或无效。");
		}
		if (!VerifyFiles(stage, manifest, true))
		{
			return failExtraction(L"释放文件校验失败。");
		}
		if (!EnsureDirectory(appDirectory))
		{
			return failExtraction(L"无法创建 App 目录。");
		}

		//A core that appeared while the payload was extracted cannot be replaced safely.
		if (IsCoreRunning(corePath))
		{
			RecallCoreWindow(corePath);
			RemoveDirectoryTree(stage, true);
			WriteDiagnosticReport(L"normal", L"blocked-core-running", false, true, appDirectory, corePath, payloadSha256, L"", 0, L"Core started during extraction; no replacement or forced close was attempted.");
			if (!suppressUi)
			{
				MessageBoxW(nullptr,
					L"RadeonTuner 正在运行，需要更新程序文件。\n\n请先从托盘菜单退出 RadeonTuner，然后重新启动此程序。\n本次启动不会强制关闭正在运行的 RadeonTuner。",
					L"RadeonTuner", MB_OK | MB_ICONINFORMATION | MB_SETFOREGROUND);
			}
			return 4;
		}

		if (!SavePending(appDirectory, payloadSha256))
		{
			return failExtraction(L"无法写入释放状态。");
		}

		bool applyOk = true;
		for (const PayloadFile& file : manifest.files)
		{
			std::wstring source = JoinPath(stage, file.relativePath);
			std::wstring destination;
			if (!PrepareSafeDestination(appDirectory, file.relativePath, destination))
			{
				applyOk = false;
				break;
			}
			//Settings\Updater.json is created only when absent. Other Settings.json, Profiles and
			//extra user files are never part of the payload and are left untouched.
			if (IsUpdaterConfigPath(file.relativePath) && FileExists(destination))
			{
				continue;
			}
			if (!MoveFileExW(source.c_str(), destination.c_str(), MOVEFILE_REPLACE_EXISTING | MOVEFILE_COPY_ALLOWED | MOVEFILE_WRITE_THROUGH))
			{
				applyOk = false;
				break;
			}
		}
		if (applyOk)
		{
			std::wstring sourceManifest = JoinPath(stage, kManifestName);
			std::wstring destinationManifest;
			applyOk = PrepareSafeDestination(appDirectory, kManifestName, destinationManifest) &&
				MoveFileExW(sourceManifest.c_str(), destinationManifest.c_str(), MOVEFILE_REPLACE_EXISTING | MOVEFILE_COPY_ALLOWED | MOVEFILE_WRITE_THROUGH) != FALSE;
		}
		if (!applyOk)
		{
			return failExtraction(L"无法覆盖 App 中的程序文件。请退出正在运行的 RadeonTuner 后重试。");
		}

		//The user-editable Updater.json is excluded from this comparison; all shipped program files
		//must still match the embedded manifest exactly.
		if (!VerifyFiles(appDirectory, manifest, false, true))
		{
			return failExtraction(L"覆盖后的程序文件校验失败。");
		}

		// A completion marker is written only after extraction, apply and a second hash validation
		// have all succeeded. If anything above failed, the pending journal remains and the partial
		// App is never launched.
		if (!SaveState(appDirectory, payloadSha256, manifest.coreSha256, false))
		{
			return failExtraction(L"无法写入完成标记。");
		}
		DeleteFileW(pendingPath.c_str());
		RemoveDirectoryTree(stage, true);

		if (IsCoreRunning(corePath))
		{
			RecallCoreWindow(corePath);
			WriteDiagnosticReport(L"normal", L"recall-existing", false, true, appDirectory, corePath, payloadSha256, L"", 0, L"Core appeared after installation; it was recalled instead of starting a second core.");
			return 0;
		}

		DWORD processId = 0;
		std::wstring commandLine;
		if (!LaunchCore(appDirectory, corePath, originalCommandLine, processId, commandLine))
		{
			WriteDiagnosticReport(L"normal", L"launch-failed", false, false, appDirectory, corePath, payloadSha256, commandLine, 0, L"CreateProcess failed after extraction: " + ErrorText(GetLastError()));
			return ShowError(L"RadeonTuner 已释放，但主程序启动失败。\n\n" + ErrorText(GetLastError()), suppressUi);
		}
		WriteDiagnosticReport(L"normal", L"extract-and-launch", true, false, appDirectory, corePath, payloadSha256, commandLine, processId, L"Payload extracted and verified before launch.");
		return 0;
	}

	// -------------------------------------------------------------------------
	// Acceptance-only extract mode. This path never creates App state and never
	// calls CreateProcess for the core application.
	// -------------------------------------------------------------------------

	int RunExtractOnly(const std::wstring& destination, const std::wstring& reportPath)
	{
		if (destination.empty() || FileExists(destination))
		{
			return 2;
		}
		WIN32_FIND_DATAW probe{};
		HANDLE probeHandle = FindFirstFileW(JoinPath(destination, L"*").c_str(), &probe);
		bool nonEmpty = false;
		if (probeHandle != INVALID_HANDLE_VALUE)
		{
			do
			{
				if (wcscmp(probe.cFileName, L".") != 0 && wcscmp(probe.cFileName, L"..") != 0)
				{
					nonEmpty = true;
					break;
				}
			} while (FindNextFileW(probeHandle, &probe));
			FindClose(probeHandle);
		}
		if (nonEmpty || !EnsureDirectory(destination))
		{
			return 2;
		}

		std::wstring finalReport = reportPath.empty() ? destination + L".extract-report.json" : reportPath;
		auto reportFailure = [&](const std::wstring& detail) -> int
		{
			std::string json = "{\n";
			json += "  \"mode\": \"extract-only\",\n";
			json += "  \"launched\": false,\n";
			json += "  \"no_launch\": true,\n";
			json += "  \"core_process_started\": false,\n";
			json += "  \"verified\": false,\n";
			json += "  \"destination\": " + JsonString(destination) + ",\n";
			json += "  \"detail\": " + JsonString(detail) + ",\n";
			json += "  \"fdi_trace\": " + JsonString(std::wstring(g_fdiTrace.begin(), g_fdiTrace.end())) + "\n";
			json += "}\n";
			WriteReport(finalReport, json);
			return 1;
		};

		if (!LoadEmbeddedPayload(GetModuleHandleW(nullptr)))
		{
			return reportFailure(L"Embedded payload resource is missing.");
		}
		std::wstring payloadSha256 = RT_PAYLOAD_ID;
		if (!IsSha256String(payloadSha256))
		{
			return reportFailure(L"Embedded payload build ID is invalid.");
		}

		if (!ExtractEmbeddedPayload(destination))
		{
			std::wstring detail = g_extractError.empty() ? L"CAB extraction failed." : std::wstring(g_extractError.begin(), g_extractError.end());
			return reportFailure(detail);
		}
		if (!VerifyManifestId(destination))
		{
			return reportFailure(L"Extracted manifest does not match the build ID.");
		}
		PayloadManifest manifest;
		if (!LoadManifest(destination, manifest))
		{
			return reportFailure(L"Extracted manifest is missing or invalid.");
		}
		if (!VerifyFiles(destination, manifest, true))
		{
			return reportFailure(L"Extracted payload verification failed.");
		}

		std::string json = "{\n";
		json += "  \"mode\": \"extract-only\",\n";
		json += "  \"launched\": false,\n";
		json += "  \"no_launch\": true,\n";
		json += "  \"core_process_started\": false,\n";
		json += "  \"verified\": true,\n";
		json += "  \"destination\": " + JsonString(destination) + ",\n";
		json += "  \"payload_sha256\": " + JsonString(payloadSha256) + ",\n";
		json += "  \"file_count\": " + JsonNumber(manifest.files.size()) + "\n";
		json += "}\n";
		WriteReport(finalReport, json);
		return 0;
	}

	// -------------------------------------------------------------------------
	// Entry point
	// -------------------------------------------------------------------------

	std::vector<std::wstring> ParseArguments()
	{
		std::vector<std::wstring> arguments;
		int count = 0;
		LPWSTR* raw = CommandLineToArgvW(GetCommandLineW(), &count);
		if (!raw)
		{
			return arguments;
		}
		for (int index = 1; index < count; index++)
		{
			arguments.emplace_back(raw[index]);
		}
		LocalFree(raw);
		return arguments;
	}
}

int APIENTRY wWinMain(HINSTANCE instance, HINSTANCE, PWSTR commandLine, int)
{
	std::wstring originalCommandLine = commandLine ? commandLine : L"";
	std::vector<std::wstring> arguments = ParseArguments();
	if (!arguments.empty() && (_wcsicmp(arguments[0].c_str(), L"--extract-only") == 0 || _wcsicmp(arguments[0].c_str(), L"-extract-only") == 0))
	{
		if (arguments.size() < 2)
		{
			return 2;
		}
		std::wstring reportPath;
		for (size_t index = 2; index + 1 < arguments.size(); index++)
		{
			if (_wcsicmp(arguments[index].c_str(), L"--extract-report") == 0)
			{
				reportPath = arguments[index + 1];
				break;
			}
		}
		return RunExtractOnly(arguments[1], reportPath);
	}
	return RunNormal(instance, originalCommandLine);
}
