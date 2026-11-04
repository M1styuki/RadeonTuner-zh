#pragma once
#include <windows.h>
#include <string>
#include <vector>
#include <fstream>
#include <filesystem>
#include <pathcch.h>
#include "AVString.h"

namespace ArnoldVinkCode
{
	inline std::string file_to_string(std::filesystem::path filePath)
	{
		try
		{
			std::ifstream file(filePath, std::ios::binary);
			if (!file.is_open())
			{
				return std::string("");
			}
			auto fileSize = std::filesystem::file_size(filePath);
			std::string string(fileSize, '\0');
			file.read(string.data(), (std::streamsize)fileSize);
			file.close();
			return string;
		}
		catch (...)
		{
			return std::string("");
		}
	}

	inline std::string file_to_string(std::string filePath)
	{
		return file_to_string(std::filesystem::path(filePath));
	}

	inline std::wstring file_to_string(std::wstring filePath)
	{
		return string_to_wstring_utf8(file_to_string(std::filesystem::path(filePath)));
	}

	inline bool string_to_file(std::filesystem::path filePath, std::string string)
	{
		try
		{
			std::ofstream file(filePath, std::ios::binary | std::ios::trunc);
			if (!file.is_open())
			{
				return false;
			}
			file.write(string.c_str(), string.size());
			file.close();
			return true;
		}
		catch (...)
		{
			return false;
		}
	}

	inline bool string_to_file(std::string filePath, std::string string)
	{
		return string_to_file(std::filesystem::path(filePath), string);
	}

	inline bool string_to_file(std::wstring filePath, std::wstring string)
	{
		return string_to_file(std::filesystem::path(filePath), wstring_to_string_utf8(string));
	}

	//Remove or add the given file attributes.
	inline bool FileAdjustAttributes(std::wstring filePath, bool removeAttributes, DWORD attributes)
	{
		try
		{
			DWORD currentAttributes = GetFileAttributesW(filePath.c_str());
			if (currentAttributes == INVALID_FILE_ATTRIBUTES)
			{
				return false;
			}

			DWORD adjustedAttributes = removeAttributes ? (currentAttributes & ~attributes) : (currentAttributes | attributes);
			if (adjustedAttributes == currentAttributes)
			{
				return true;
			}
			return SetFileAttributesW(filePath.c_str(), adjustedAttributes) != false;
		}
		catch (...)
		{
			return false;
		}
	}

	//Check if a file can be created in the target folder without keeping it.
	inline bool FolderWritePermission(std::wstring folderPath)
	{
		try
		{
			std::filesystem::path targetFolder = folderPath.empty() ? std::filesystem::current_path() : std::filesystem::path(folderPath);
			std::filesystem::path testFilePath = targetFolder / (L".RadeonTunerWriteTest_" + std::to_wstring(GetCurrentProcessId()) + L"_" + std::to_wstring(GetTickCount64()) + L".tmp");
			HANDLE testFile = CreateFileW(testFilePath.c_str(), GENERIC_WRITE, 0, NULL, CREATE_ALWAYS, FILE_ATTRIBUTE_TEMPORARY | FILE_FLAG_DELETE_ON_CLOSE, NULL);
			if (testFile == INVALID_HANDLE_VALUE)
			{
				return false;
			}
			CloseHandle(testFile);
			return true;
		}
		catch (...)
		{
			return false;
		}
	}

	inline bool FileDelete(std::wstring filePath)
	{
		try
		{
			return std::filesystem::remove(filePath);
		}
		catch (...)
		{
			return false;
		}
	}

	inline bool FileExists(std::wstring filePath)
	{
		try
		{
			return std::filesystem::exists(filePath);
		}
		catch (...)
		{
			return false;
		}
	}

	inline bool FileMove(std::wstring oldFilePath, std::wstring newFilePath, bool overWrite)
	{
		try
		{
			if (oldFilePath == newFilePath)
			{
				AVDebugWriteLine(L"Failed moving file: targeting the same path.");
				return false;
			}

			if (FileExists(oldFilePath))
			{
				AVDebugWriteLine(L"Moving: " + oldFilePath + L" to " + newFilePath);
				if (overWrite) { FileDelete(newFilePath); }
				std::filesystem::rename(oldFilePath, newFilePath);
				return true;
			}
			else
			{
				AVDebugWriteLine(L"Failed moving file: " + oldFilePath + L" does not exist");
				return false;
			}
		}
		catch (...)
		{
			return false;
		}
	}

	inline std::vector<std::filesystem::directory_entry> FileList(std::wstring folderPath, bool listRecursive)
	{
		std::vector<std::filesystem::directory_entry> fileList;
		try
		{
			if (listRecursive)
			{
				auto iterator = std::filesystem::recursive_directory_iterator(folderPath);
				for (auto entry : iterator)
				{
					fileList.push_back(entry);
				}
			}
			else
			{
				auto iterator = std::filesystem::directory_iterator(folderPath);
				for (auto entry : iterator)
				{
					fileList.push_back(entry);
				}
			}
		}
		catch (...) {}
		return fileList;
	}

	inline bool FolderCreate(std::wstring folderPath)
	{
		try
		{
			return std::filesystem::create_directory(folderPath);
		}
		catch (...)
		{
			return false;
		}
	}

	inline bool FolderExists(std::wstring folderPath)
	{
		try
		{
			return std::filesystem::exists(folderPath);
		}
		catch (...)
		{
			return false;
		}
	}

	inline bool FolderDelete(std::wstring folderPath, bool recursive)
	{
		try
		{
			if (!std::filesystem::exists(folderPath))
			{
				return false;
			}
			if (recursive)
			{
				return std::filesystem::remove_all(folderPath) > 0;
			}
			return std::filesystem::remove(folderPath);
		}
		catch (...)
		{
			return false;
		}
	}
}