// Scope: the background process-name matching step only. This benchmark does not measure
// the application's resident working set, UI, ADL/ADLX, or any GPU code.
//
// Old API: vendored ArnoldVinkCode::array_contains(std::vector<std::wstring> array,
//          std::wstring value), which copies the vector and query for every call.
// Current API: std::find(fileExecutables.begin(), fileExecutables.end(), fileExtension),
//          using const references and no copy.

#ifndef WIN32_LEAN_AND_MEAN
#define WIN32_LEAN_AND_MEAN
#endif
#ifndef NOMINMAX
#define NOMINMAX
#endif

#include <windows.h>

#include <algorithm>
#include <atomic>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cwchar>
#include <fstream>
#include <new>
#include <string>
#include <vector>

#include "../../third_party/ArnoldVinkCode/CPP/AVLinq.h"

namespace
{
	std::atomic<bool> g_measureActive{ false };
	std::atomic<unsigned long long> g_newCalls{ 0 };
	std::atomic<unsigned long long> g_newBytes{ 0 };

	void RecordAllocation(std::size_t size)
	{
		if (g_measureActive.load(std::memory_order_relaxed))
		{
			g_newCalls.fetch_add(1, std::memory_order_relaxed);
			g_newBytes.fetch_add(size, std::memory_order_relaxed);
		}
	}
}

void* operator new(std::size_t size)
{
	RecordAllocation(size);
	void* pointer = std::malloc(size == 0 ? 1 : size);
	if (!pointer)
	{
		throw std::bad_alloc();
	}
	return pointer;
}

void* operator new[](std::size_t size)
{
	RecordAllocation(size);
	void* pointer = std::malloc(size == 0 ? 1 : size);
	if (!pointer)
	{
		throw std::bad_alloc();
	}
	return pointer;
}

void operator delete(void* pointer) noexcept
{
	std::free(pointer);
}

void operator delete[](void* pointer) noexcept
{
	std::free(pointer);
}

void operator delete(void* pointer, std::size_t) noexcept
{
	std::free(pointer);
}

void operator delete[](void* pointer, std::size_t) noexcept
{
	std::free(pointer);
}

namespace
{
	struct Measurement
	{
		unsigned long long NewCalls = 0;
		unsigned long long NewBytes = 0;
		unsigned long long ElapsedNanoseconds = 0;
		std::uint64_t Checksum = 0;
	};

	unsigned long long QueryTicks()
	{
		LARGE_INTEGER value{};
		QueryPerformanceCounter(&value);
		return (unsigned long long)value.QuadPart;
	}

	unsigned long long QueryFrequency()
	{
		LARGE_INTEGER value{};
		QueryPerformanceFrequency(&value);
		return (unsigned long long)value.QuadPart;
	}

	std::uint64_t Mix(std::uint64_t checksum, bool found, std::size_t index)
	{
		std::uint64_t value = found ? 0x9E3779B97F4A7C15ull : 0xC2B2AE3D27D4EB4Full;
		value ^= (std::uint64_t)index * 0x100000001B3ull;
		return (checksum ^ value) * 0x100000001B3ull;
	}

	//A deterministic process-name sequence: common launchers, uppercase extensions that the
	//caller lowercases, unknown extensions, and repeated names.
	std::vector<std::wstring> BuildProcessNames()
	{
		const wchar_t* fixedNames[] =
		{
			L"RadeonTuner.exe", L"gamelauncher.exe", L"steam.exe", L"browser.EXE",
			L"tool.dll", L"batch.CMD", L"setup.com", L"script.bat", L"payload.bin",
			L"readme.txt", L"helper.pif", L"no-extension", L"trailing.exe.txt",
			L"RadeonTuner.exe", L"game.exe"
		};
		std::vector<std::wstring> names;
		names.reserve(64);
		for (const wchar_t* name : fixedNames)
		{
			names.emplace_back(name);
		}
		for (int index = 0; names.size() < 64; index++)
		{
			names.push_back(L"worker-" + std::to_wstring(index) + (index % 3 == 0 ? L".exe" : (index % 3 == 1 ? L".com" : L".dll")));
		}
		return names;
	}

	std::vector<std::wstring> BuildQueries(const std::vector<std::wstring>& processNames)
	{
		std::vector<std::wstring> queries;
		queries.reserve(processNames.size());
		for (const std::wstring& name : processNames)
		{
			size_t dot = name.find_last_of(L'.');
			if (dot == std::wstring::npos)
			{
				queries.emplace_back(L"");
				continue;
			}
			std::wstring extension = name.substr(dot);
			for (wchar_t& character : extension)
			{
				if (character >= L'A' && character <= L'Z')
				{
					character = (wchar_t)(character - L'A' + L'a');
				}
			}
			queries.push_back(extension);
		}
		return queries;
	}

	Measurement MeasureOldApi(const std::vector<std::wstring>& fileExecutables, const std::vector<std::wstring>& queries, int repetitions)
	{
		Measurement result;
		for (int repeat = 0; repeat < repetitions; repeat++)
		{
			g_newCalls.store(0, std::memory_order_relaxed);
			g_newBytes.store(0, std::memory_order_relaxed);
			std::uint64_t checksum = 0xCBF29CE484222325ull;
			g_measureActive.store(true, std::memory_order_relaxed);
			unsigned long long start = QueryTicks();
			for (int round = 0; round < 30000; round++)
			{
				for (std::size_t index = 0; index < queries.size(); index++)
				{
					bool found = ArnoldVinkCode::array_contains(fileExecutables, queries[index]);
					checksum = Mix(checksum, found, index);
				}
			}
			unsigned long long end = QueryTicks();
			g_measureActive.store(false, std::memory_order_relaxed);
			if (repeat == 0)
			{
				result.Checksum = checksum;
			}
			else if (result.Checksum != checksum)
			{
				result.Checksum = 0;
			}
			if (repeat == 0 || g_newCalls.load() < result.NewCalls)
			{
				result.NewCalls = g_newCalls.load();
				result.NewBytes = g_newBytes.load();
			}
			unsigned long long elapsed = (end - start) * 1000000000ull / QueryFrequency();
			if (repeat == 0 || elapsed < result.ElapsedNanoseconds)
			{
				result.ElapsedNanoseconds = elapsed;
			}
		}
		return result;
	}

	Measurement MeasureCurrentApi(const std::vector<std::wstring>& fileExecutables, const std::vector<std::wstring>& queries, int repetitions)
	{
		Measurement result;
		for (int repeat = 0; repeat < repetitions; repeat++)
		{
			g_newCalls.store(0, std::memory_order_relaxed);
			g_newBytes.store(0, std::memory_order_relaxed);
			std::uint64_t checksum = 0xCBF29CE484222325ull;
			g_measureActive.store(true, std::memory_order_relaxed);
			unsigned long long start = QueryTicks();
			for (int round = 0; round < 30000; round++)
			{
				for (std::size_t index = 0; index < queries.size(); index++)
				{
					bool found = std::find(fileExecutables.begin(), fileExecutables.end(), queries[index]) != fileExecutables.end();
					checksum = Mix(checksum, found, index);
				}
			}
			unsigned long long end = QueryTicks();
			g_measureActive.store(false, std::memory_order_relaxed);
			if (repeat == 0)
			{
				result.Checksum = checksum;
			}
			else if (result.Checksum != checksum)
			{
				result.Checksum = 0;
			}
			if (repeat == 0 || g_newCalls.load() < result.NewCalls)
			{
				result.NewCalls = g_newCalls.load();
				result.NewBytes = g_newBytes.load();
			}
			unsigned long long elapsed = (end - start) * 1000000000ull / QueryFrequency();
			if (repeat == 0 || elapsed < result.ElapsedNanoseconds)
			{
				result.ElapsedNanoseconds = elapsed;
			}
		}
		return result;
	}

	std::string JsonEscape(const std::string& value)
	{
		std::string result;
		for (unsigned char character : value)
		{
			switch (character)
			{
			case '"': result += "\\\""; break;
			case '\\': result += "\\\\"; break;
			case '\n': result += "\\n"; break;
			case '\r': result += "\\r"; break;
			default:
				if (character < 0x20)
				{
					char buffer[8]{};
					sprintf_s(buffer, sizeof(buffer), "\\u%04x", character);
					result += buffer;
				}
				else
				{
					result.push_back((char)character);
				}
				break;
			}
		}
		return result;
	}

	std::string WideToUtf8(const std::wstring& value)
	{
		int length = WideCharToMultiByte(CP_UTF8, 0, value.data(), (int)value.size(), nullptr, 0, nullptr, nullptr);
		if (length <= 0)
		{
			return std::string();
		}
		std::string result(length, '\0');
		WideCharToMultiByte(CP_UTF8, 0, value.data(), (int)value.size(), result.data(), length, nullptr, nullptr);
		return result;
	}

	std::string Number(unsigned long long value)
	{
		return std::to_string(value);
	}
}

int wmain(int argumentCount, wchar_t** arguments)
{
	std::wstring jsonPath;
	std::wstring csvPath;
	for (int index = 1; index < argumentCount; index++)
	{
		if (_wcsicmp(arguments[index], L"--out") == 0 && index + 1 < argumentCount)
		{
			jsonPath = arguments[++index];
		}
		else if (_wcsicmp(arguments[index], L"--csv") == 0 && index + 1 < argumentCount)
		{
			csvPath = arguments[++index];
		}
	}

	std::vector<std::wstring> fileExecutables = { L".exe", L".bat", L".cmd", L".com", L".pif", L".bin" };
	std::vector<std::wstring> processNames = BuildProcessNames();
	std::vector<std::wstring> queries = BuildQueries(processNames);
	const int repetitions = 3;
	const unsigned long long callsPerRun = (unsigned long long)queries.size() * 30000ull;

	//Warm up both paths before the counted windows.
	for (const std::wstring& query : queries)
	{
		ArnoldVinkCode::array_contains(fileExecutables, query);
		(void)std::find(fileExecutables.begin(), fileExecutables.end(), query);
	}

	Measurement oldResult = MeasureOldApi(fileExecutables, queries, repetitions);
	Measurement newResult = MeasureCurrentApi(fileExecutables, queries, repetitions);

	std::string json = "{\n";
	json += "  \"scope\": \"background process-name matching step only\",\n";
	json += "  \"not_measured\": [\"resident working set\", \"UI memory\", \"ADL/ADLX\", \"GPU code\", \"whole application\"],\n";
	json += "  \"old_api\": \"ArnoldVinkCode::array_contains(std::vector<std::wstring> by value, std::wstring by value)\",\n";
	json += "  \"current_api\": \"std::find(fileExecutables.begin(), fileExecutables.end(), fileExtension)\",\n";
	json += "  \"compiler_optimization\": \"O2\",\n";
	json += "  \"measurement\": \"operator new calls/bytes actually requested while the API loop runs; QueryPerformanceCounter elapsed; best of 3\",\n";
	json += "  \"file_executables\": [";
	for (std::size_t index = 0; index < fileExecutables.size(); index++)
	{
		if (index > 0) json += ", ";
		json += "\"" + JsonEscape(WideToUtf8(fileExecutables[index])) + "\"";
	}
	json += "],\n";
	json += "  \"query_count\": " + Number(queries.size()) + ",\n";
	json += "  \"rounds\": 30000,\n";
	json += "  \"calls_per_run\": " + Number(callsPerRun) + ",\n";
	json += "  \"query_sequence\": [";
	for (std::size_t index = 0; index < queries.size(); index++)
	{
		if (index > 0) json += ", ";
		json += "\"" + JsonEscape(WideToUtf8(queries[index])) + "\"";
	}
	json += "],\n";
	json += "  \"old_array_contains\": {\n";
	json += "    \"operator_new_calls\": " + Number(oldResult.NewCalls) + ",\n";
	json += "    \"operator_new_bytes\": " + Number(oldResult.NewBytes) + ",\n";
	json += "    \"elapsed_ns\": " + Number(oldResult.ElapsedNanoseconds) + ",\n";
	json += "    \"checksum\": " + Number(oldResult.Checksum) + "\n";
	json += "  },\n";
	json += "  \"current_std_find\": {\n";
	json += "    \"operator_new_calls\": " + Number(newResult.NewCalls) + ",\n";
	json += "    \"operator_new_bytes\": " + Number(newResult.NewBytes) + ",\n";
	json += "    \"elapsed_ns\": " + Number(newResult.ElapsedNanoseconds) + ",\n";
	json += "    \"checksum\": " + Number(newResult.Checksum) + "\n";
	json += "  },\n";
	json += "  \"results_identical\": " + std::string(oldResult.Checksum == newResult.Checksum && oldResult.Checksum != 0 ? "true" : "false") + ",\n";
	json += "  \"allocation_requests_avoided\": " + Number(oldResult.NewCalls > newResult.NewCalls ? oldResult.NewCalls - newResult.NewCalls : 0) + ",\n";
	json += "  \"allocation_bytes_avoided\": " + Number(oldResult.NewBytes > newResult.NewBytes ? oldResult.NewBytes - newResult.NewBytes : 0) + ",\n";
	json += "  \"statement\": \"These counts cover only the matching function calls, not the application process working set.\"\n";
	json += "}\n";

	if (!jsonPath.empty())
	{
		std::ofstream output(jsonPath, std::ios::binary | std::ios::trunc);
		output << json;
		output.close();
	}
	std::fwrite(json.data(), 1, json.size(), stdout);

	if (!csvPath.empty())
	{
		std::ofstream output(csvPath, std::ios::binary | std::ios::trunc);
		output << "api,operator_new_calls,operator_new_bytes,elapsed_ns,checksum\n";
		output << "old_array_contains," << oldResult.NewCalls << "," << oldResult.NewBytes << "," << oldResult.ElapsedNanoseconds << "," << oldResult.Checksum << "\n";
		output << "current_std_find," << newResult.NewCalls << "," << newResult.NewBytes << "," << newResult.ElapsedNanoseconds << "," << newResult.Checksum << "\n";
		output.close();
	}

	bool valid = oldResult.Checksum != 0 && oldResult.Checksum == newResult.Checksum && oldResult.NewCalls > newResult.NewCalls;
	return valid ? 0 : 1;
}
