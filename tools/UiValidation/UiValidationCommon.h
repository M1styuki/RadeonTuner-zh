#pragma once
#include <Windows.h>
#include <Psapi.h>
#include <chrono>
#include <cstdio>
#include <filesystem>
#include <fstream>
#include <iomanip>
#include <mutex>
#include <sstream>
#include <string>
#include <vector>

#pragma comment(lib, "Psapi.lib")

//Shared by the isolated UI validation entry point and scenario.
//All data written here belongs under the test build's own UiValidationData folder.
namespace RadeonTunerUiValidation
{
	inline std::wstring SomeText(winrt::Windows::Foundation::IInspectable const& value)
	{
		if (!value) { return L""; }
		auto text = value.try_as<winrt::hstring>();
		return text.has_value() ? std::wstring(text.value().c_str()) : L"";
	}

	inline std::wstring SomeText(winrt::hstring const& value)
	{
		return std::wstring(value.c_str());
	}

	inline void Require(bool condition, std::wstring const& message)
	{
		if (!condition)
		{
			throw winrt::hresult_error(E_FAIL, winrt::hstring(message));
		}
	}

	inline bool AutoMode = true;
	inline bool PreflightOnly = false;
	inline bool ShowTrayMenuOnly = false;
	inline bool PreviewMode = false;
	inline bool AnimationCheckMode = false;
	inline bool LifecycleCheckMode = false;
	inline bool LifecycleCheckStarted = false;
	inline int LifecycleCyclesCompleted = 0;
	inline bool LifecycleOldPageReleasedVerified = false;
	inline bool LifecycleTimersStoppedVerified = false;
	inline bool LifecycleBackendNotDuplicatedVerified = false;
	inline bool LifecycleDraftRestoredVerified = false;
	inline bool LifecycleTrayRecallVerified = false;
	inline bool LifecycleExeRecallVerified = false;
	inline bool LifecycleRapidReopenVerified = false;
	inline bool AnimationCheckStay = false;
	inline bool UnicodePathTestsPassed = false;
	inline bool SystemIntegrationTestsPassed = false;
	inline bool ScenarioStarted = false;
	inline bool ScenarioCompleted = false;
	inline bool ScenarioFailed = false;
	inline bool CrossProcessRecallVerified = false;
	inline bool EnumListsPrepared = false;
	inline bool ThemeSwitchVerified = false;
	inline bool UiCompatibilityVerified = false;
	inline bool FlatLayoutVerified = false;
	inline int FlatRowsVerified = 0;
	inline int CrossProcessRecallChildExitCode = -1;
	inline uint64_t CrossProcessRecallMilliseconds = 0;
	inline long ScenarioFailureHresult = 0;
	inline std::wstring ScenarioFailureContext;
	inline std::wstring ScenarioFailureMessage;
	inline std::wstring TestDataPath;
	inline std::wstring ExecutablePath;
	inline std::wstring MeasurementsCsvPath;
	inline std::wstring ResultJsonPath;
	inline std::wstring EnvironmentJsonPath;
	inline std::wstring PreflightJsonPath;
	inline std::wstring DebugLogPath;
	inline int PagesNavigated = 0;
	inline int PopupRoundsCompleted = 0;
	inline int RealMessageBoxRoundsCompleted = 0;
	inline int TrayToggleRoundsCompleted = 0;
	inline bool TrayDefaultFalseVerified = false;
	inline bool TrayPersistedVerified = false;
	inline bool TrayReloadedVerified = false;
	inline bool TrayHiddenVerified = false;
	inline bool TrayHiddenAfterTaskbarRecreatedVerified = false;
	inline bool TrayRestoredVerified = false;
	inline bool TrayVisibleAfterTaskbarRecreatedVerified = false;
	inline bool TrayThemeChangeWhileHiddenVerified = false;
	inline int TrayUserObjectDelta = 0;
	inline int TrayHandleCountDelta = 0;
	inline std::chrono::steady_clock::time_point ScenarioStart;
	inline std::ofstream MeasurementFile;
	inline std::mutex MeasurementMutex;

	inline std::string WideToUtf8(std::wstring const& value)
	{
		if (value.empty())
		{
			return std::string();
		}
		int byteCount = WideCharToMultiByte(CP_UTF8, 0, value.data(), (int)value.size(), NULL, 0, NULL, NULL);
		if (byteCount <= 0)
		{
			return std::string();
		}
		std::string result(byteCount, '\0');
		WideCharToMultiByte(CP_UTF8, 0, value.data(), (int)value.size(), result.data(), byteCount, NULL, NULL);
		return result;
	}

	inline std::wstring Utf8ToWide(std::string const& value)
	{
		if (value.empty())
		{
			return std::wstring();
		}
		int charCount = MultiByteToWideChar(CP_UTF8, 0, value.data(), (int)value.size(), NULL, 0);
		if (charCount <= 0)
		{
			return std::wstring();
		}
		std::wstring result(charCount, L'\0');
		MultiByteToWideChar(CP_UTF8, 0, value.data(), (int)value.size(), result.data(), charCount);
		return result;
	}

	inline std::string JsonEscape(std::string const& value)
	{
		std::string result;
		result.reserve(value.size() + 16);
		for (unsigned char character : value)
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
					char buffer[8] = {};
					sprintf_s(buffer, sizeof(buffer), "\\u%04X", character);
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

	inline std::string JsonString(std::wstring const& value)
	{
		return "\"" + JsonEscape(WideToUtf8(value)) + "\"";
	}

	inline std::string JsonString(std::string const& value)
	{
		return "\"" + JsonEscape(value) + "\"";
	}

	inline std::string HresultToString(long hresult)
	{
		char buffer[16] = {};
		sprintf_s(buffer, sizeof(buffer), "0x%08lX", (unsigned long)hresult);
		return std::string(buffer);
	}

	inline std::string Win32ErrorToString(DWORD errorCode)
	{
		char buffer[32] = {};
		sprintf_s(buffer, sizeof(buffer), "0x%08lX", (unsigned long)errorCode);
		return std::string(buffer);
	}

	inline std::wstring Win32ErrorMessage(DWORD errorCode)
	{
		LPWSTR buffer = NULL;
		DWORD length = FormatMessageW(FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS, NULL, errorCode, 0, (LPWSTR)&buffer, 0, NULL);
		if (length == 0 || buffer == NULL)
		{
			return L"Unknown error";
		}
		std::wstring message(buffer, length);
		LocalFree(buffer);
		return message;
	}

	inline std::string UtcNowString()
	{
		SYSTEMTIME systemTime = {};
		GetSystemTime(&systemTime);
		char buffer[32] = {};
		sprintf_s(buffer, sizeof(buffer), "%04u-%02u-%02uT%02u:%02u:%02uZ", systemTime.wYear, systemTime.wMonth, systemTime.wDay, systemTime.wHour, systemTime.wMinute, systemTime.wSecond);
		return std::string(buffer);
	}

	inline bool WriteUtf8File(std::wstring const& filePath, std::string const& text)
	{
		HANDLE fileHandle = CreateFileW(filePath.c_str(), GENERIC_WRITE, FILE_SHARE_READ, NULL, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, NULL);
		if (fileHandle == INVALID_HANDLE_VALUE)
		{
			return false;
		}
		DWORD bytesWritten = 0;
		BOOL writeResult = WriteFile(fileHandle, text.data(), (DWORD)text.size(), &bytesWritten, NULL);
		CloseHandle(fileHandle);
		return writeResult != FALSE && bytesWritten == text.size();
	}

	inline uint64_t ElapsedSinceStartMilliseconds()
	{
		return (uint64_t)std::chrono::duration_cast<std::chrono::milliseconds>(std::chrono::steady_clock::now() - ScenarioStart).count();
	}

	struct ProcessMetrics
	{
		uint64_t PrivateBytes = 0;
		uint64_t WorkingSetBytes = 0;
		uint32_t HandleCount = 0;
		uint32_t GdiObjects = 0;
		uint32_t UserObjects = 0;
	};

	inline ProcessMetrics SampleProcessMetrics()
	{
		ProcessMetrics metrics;
		PROCESS_MEMORY_COUNTERS_EX memoryCounters = {};
		memoryCounters.cb = sizeof(memoryCounters);
		if (GetProcessMemoryInfo(GetCurrentProcess(), (PROCESS_MEMORY_COUNTERS*)&memoryCounters, sizeof(memoryCounters)))
		{
			metrics.PrivateBytes = (uint64_t)memoryCounters.PrivateUsage;
			metrics.WorkingSetBytes = (uint64_t)memoryCounters.WorkingSetSize;
		}
		DWORD handleCount = 0;
		if (GetProcessHandleCount(GetCurrentProcess(), &handleCount))
		{
			metrics.HandleCount = handleCount;
		}
		metrics.GdiObjects = GetGuiResources(GetCurrentProcess(), GR_GDIOBJECTS);
		metrics.UserObjects = GetGuiResources(GetCurrentProcess(), GR_USEROBJECTS);
		return metrics;
	}

	inline void PumpMessagesFor(int milliseconds)
	{
		ULONGLONG endTime = GetTickCount64() + (ULONGLONG)milliseconds;
		do
		{
			MSG message = {};
			while (PeekMessageW(&message, NULL, 0, 0, PM_REMOVE))
			{
				TranslateMessage(&message);
				DispatchMessageW(&message);
			}
			Sleep(5);
		} while (GetTickCount64() < endTime);
	}

	inline std::string CsvEscape(std::string const& value)
	{
		if (value.find_first_of(",\"\r\n") == std::string::npos)
		{
			return value;
		}
		std::string result = "\"";
		for (char character : value)
		{
			if (character == '"')
			{
				result += "\"\"";
			}
			else
			{
				result.push_back(character);
			}
		}
		result += "\"";
		return result;
	}

	inline void OpenMeasurements()
	{
		if (MeasurementsCsvPath.empty())
		{
			MeasurementsCsvPath = TestDataPath + L"\\ui-validation-measurements.csv";
		}
		std::lock_guard<std::mutex> lock(MeasurementMutex);
		MeasurementFile.open(std::filesystem::path(MeasurementsCsvPath), std::ios::binary | std::ios::trunc);
		if (MeasurementFile.is_open())
		{
			MeasurementFile << "\xEF\xBB\xBF";
			MeasurementFile << "stage,round,page,elapsed_ms,private_bytes,working_set_bytes,handle_count,gdi_objects,user_objects,window_visible,detail\n";
			MeasurementFile.flush();
		}
	}

	inline void CloseMeasurements()
	{
		std::lock_guard<std::mutex> lock(MeasurementMutex);
		if (MeasurementFile.is_open())
		{
			MeasurementFile.flush();
			MeasurementFile.close();
		}
	}

	inline void AppendMeasurement(std::string const& stage, int round, int page, uint64_t elapsedMilliseconds, ProcessMetrics const& metrics, bool windowVisible, std::wstring const& detail)
	{
		std::lock_guard<std::mutex> lock(MeasurementMutex);
		if (!MeasurementFile.is_open())
		{
			return;
		}
		MeasurementFile << CsvEscape(stage) << ","
			<< round << ","
			<< page << ","
			<< elapsedMilliseconds << ","
			<< metrics.PrivateBytes << ","
			<< metrics.WorkingSetBytes << ","
			<< metrics.HandleCount << ","
			<< metrics.GdiObjects << ","
			<< metrics.UserObjects << ","
			<< (windowVisible ? 1 : 0) << ","
			<< CsvEscape(WideToUtf8(detail))
			<< "\n";
		MeasurementFile.flush();
	}

	inline void RecordInitFailure(long hresult, std::wstring const& context, std::wstring const& message)
	{
		ScenarioFailed = true;
		ScenarioFailureHresult = hresult;
		ScenarioFailureContext = context;
		ScenarioFailureMessage = message;
		if (!TestDataPath.empty())
		{
			std::string content;
			content += "context=" + WideToUtf8(context) + "\n";
			content += "hresult=" + HresultToString(hresult) + "\n";
			content += "message=" + WideToUtf8(message) + "\n";
			content += "win32_error=" + Win32ErrorToString(GetLastError()) + "\n";
			WriteUtf8File(TestDataPath + L"\\ui-validation-init-failure.txt", content);
		}
	}

	inline void RecordScenarioFailure(long hresult, std::wstring const& context, std::wstring const& message)
	{
		ScenarioFailed = true;
		ScenarioFailureHresult = hresult;
		ScenarioFailureContext = context;
		ScenarioFailureMessage = message;
	}
}
