#pragma once
#include "pch.h"
#include <cstdint>
#include <filesystem>
#include <fstream>
#include <string>
#include <vector>
#include "SteamFuncLibraryFolders.h"
#include "SteamFuncAppInfo.h"
#include "UiValidationCommon.h"

namespace RadeonTunerUiValidation::UnicodePathTests
{
	inline void AppendU32(std::vector<uint8_t>& data, uint32_t value)
	{
		for (int index = 0; index < 4; index++)
		{
			data.push_back((uint8_t)(value >> (index * 8)));
		}
	}

	inline void AppendU64(std::vector<uint8_t>& data, uint64_t value)
	{
		for (int index = 0; index < 8; index++)
		{
			data.push_back((uint8_t)(value >> (index * 8)));
		}
	}

	inline void AppendString(std::vector<uint8_t>& data, std::string const& value)
	{
		data.insert(data.end(), value.begin(), value.end());
		data.push_back(0);
	}

	inline std::vector<uint8_t> BuildAppInfoFile(std::string const& appName, std::string const& executable)
	{
		//String table used by the minimal version 41 appinfo record
		std::vector<std::string> stringTable = { "appinfo", "common", "name", "config", "launch", "0", "executable" };

		//Build the application VDF data
		std::vector<uint8_t> vdf;
		auto appendKey = [&](uint8_t keyType, uint32_t keyIndex)
		{
			vdf.push_back(keyType);
			AppendU32(vdf, keyIndex);
		};

		appendKey(0x00, 0); //appinfo
		appendKey(0x00, 1); //common
		appendKey(0x01, 2); //name
		AppendString(vdf, appName);
		vdf.push_back(0x08);
		appendKey(0x00, 3); //config
		appendKey(0x00, 4); //launch
		appendKey(0x00, 5); //0
		appendKey(0x01, 6); //executable
		AppendString(vdf, executable);
		vdf.push_back(0x08);
		vdf.push_back(0x08);
		vdf.push_back(0x08);
		vdf.push_back(0x08);

		//Build the appinfo file
		std::vector<uint8_t> file;
		AppendU32(file, 41); //magic/version
		AppendU32(file, 1); //universe
		size_t stringTableOffsetPosition = file.size();
		AppendU64(file, 0); //string table offset placeholder
		AppendU32(file, 1); //app identifier
		AppendU32(file, (uint32_t)(60 + vdf.size())); //app data size
		AppendU32(file, 1); //app info state
		AppendU32(file, 0); //last updated
		AppendU64(file, 0); //token
		for (int index = 0; index < 20; index++) { file.push_back(0); } //sha1 hash
		AppendU32(file, 0); //change number
		for (int index = 0; index < 20; index++) { file.push_back(0); } //sha1 binary
		file.insert(file.end(), vdf.begin(), vdf.end());
		AppendU32(file, 0); //end of app list

		//Append string table and patch its offset
		uint64_t stringTableOffset = file.size();
		for (int index = 0; index < 8; index++)
		{
			file[stringTableOffsetPosition + index] = (uint8_t)(stringTableOffset >> (index * 8));
		}
		AppendU32(file, (uint32_t)stringTable.size());
		for (std::string const& stringValue : stringTable)
		{
			AppendString(file, stringValue);
		}
		return file;
	}

	inline bool Run(std::wstring testDataPath)
	{
		nlohmann::json report = nlohmann::json::object();
		report["started_utc"] = UtcNowString();
		report["cases"] = nlohmann::json::array();
		bool allPassed = true;

		auto addCase = [&](std::string const& name, bool passed, std::wstring const& detail)
		{
			allPassed = allPassed && passed;
			nlohmann::json testCase = nlohmann::json::object();
			testCase["name"] = name;
			testCase["passed"] = passed;
			testCase["detail"] = WideToUtf8(detail);
			report["cases"].push_back(testCase);
			AVDebugWriteLine(L"Unicode path test: " + Utf8ToWide(name) + (passed ? L" passed." : L" failed."));
		};

		std::filesystem::path testRoot = std::filesystem::path(testDataPath) / L"UnicodePathTests";
		report["test_root"] = WideToUtf8(testRoot.wstring());
		try
		{
			//Only remove the test-owned temporary folder
			std::error_code removeError;
			std::filesystem::remove_all(testRoot, removeError);
			std::filesystem::create_directories(testRoot);

			std::wstring chineseFolder = L"\u4E2D\u6587\u8DEF\u5F84";
			std::wstring accentedFolder = L"caf\u00E9-\u00E9\u00E8-\u00FC";
			std::filesystem::path unicodeFolder = testRoot / chineseFolder / accentedFolder;
			std::filesystem::create_directories(unicodeFolder);

			//Actual UTF-8 read/write through a wide filesystem path
			{
				std::filesystem::path filePath = unicodeFolder / L"\u6587\u4EF6-\u00E9.txt";
				std::string content = WideToUtf8(L"\u5185\u5BB9 UTF-8 \u00E9\u00E8\u00FC");
				bool writeResult = ArnoldVinkCode::string_to_file(std::filesystem::path(filePath), content);
				std::string readContent = ArnoldVinkCode::file_to_string(std::filesystem::path(filePath));
				std::ifstream directStream(filePath, std::ios::binary);
				bool directOpen = directStream.is_open();
				addCase("utf8_read_write_unicode_directory", writeResult && readContent == content && directOpen, L"path=" + filePath.wstring());
			}

			//Steam libraryfolders.vdf must open and parse through a wide path
			{
				std::filesystem::path steamRoot = testRoot / accentedFolder / L"Steam";
				std::filesystem::path steamApps = steamRoot / L"steamapps";
				std::filesystem::create_directories(steamApps);
				std::filesystem::path libraryFoldersPath = steamApps / L"libraryfolders.vdf";
				std::filesystem::path libraryPath = testRoot / chineseFolder / L"SteamLibrary";
				std::wstring libraryPathText = libraryPath.wstring();
				std::wstring vdfText = L"\"libraryfolders\"\n{\n\t\"0\"\n\t{\n\t\t\"path\" \"" + libraryPathText + L"\"\n\t\t\"apps\"\n\t\t{\n\t\t\t\"12345\" \"999\"\n\t\t}\n\t}\n}\n";
				bool writeResult = ArnoldVinkCode::string_to_file(std::filesystem::path(libraryFoldersPath), WideToUtf8(vdfText));
				std::vector<LauncherSteam::Structures::SteamLibraryFoldersBasic> folders = LauncherSteam::LibraryFolders::GetLibraryFolders(std::filesystem::path(libraryFoldersPath));
				bool folderFound = folders.size() == 1;
				bool folderPathMatches = folderFound && folders[0].path == WideToUtf8(libraryPathText);
				bool appsFound = folderFound && folders[0].apps.has_value() && folders[0].apps.value().size() == 1 && folders[0].apps.value()[0] == "12345";
				addCase("steam_libraryfolders_wide_path", writeResult && folderFound && folderPathMatches && appsFound, L"path=" + libraryFoldersPath.wstring() + L"; library=" + libraryPathText);
			}

			//Steam appinfo.vdf must open and parse through a wide path
			{
				std::filesystem::path appInfoFolder = testRoot / chineseFolder / L"appcache";
				std::filesystem::create_directories(appInfoFolder);
				std::filesystem::path appInfoPath = appInfoFolder / L"appinfo.vdf";
				std::wstring appName = L"\u6D4B\u8BD5\u6E38\u620F-\u00E9";
				std::vector<uint8_t> appInfoData = BuildAppInfoFile(WideToUtf8(appName), "game.exe");
				std::ofstream appInfoStream(appInfoPath, std::ios::binary | std::ios::trunc);
				appInfoStream.write((const char*)appInfoData.data(), (std::streamsize)appInfoData.size());
				appInfoStream.close();

				std::vector<LauncherSteam::Structures::SteamAppInfoBasic> apps = LauncherSteam::AppInfo::GetApps(std::filesystem::path(appInfoPath));
				bool appFound = apps.size() == 1;
				bool appNameMatches = appFound && apps[0].name.has_value() && apps[0].name.value() == WideToUtf8(appName);
				bool executableMatches = appFound && apps[0].executable.has_value() && apps[0].executable.value() == "game.exe";
				addCase("steam_appinfo_wide_path", appFound && appNameMatches && executableMatches, L"path=" + appInfoPath.wstring() + L"; app=" + appName);
			}

			//Startup shortcut keeps the .url naming convention and stores a Unicode target
			{
				std::filesystem::path shortcutFolder = testRoot / chineseFolder / accentedFolder / L"shortcuts";
				std::filesystem::create_directories(shortcutFolder);
				std::wstring appName = L"RadeonTuner-\u6D4B\u8BD5-\u00E9";
				std::wstring targetPath = (shortcutFolder / L"\u5E94\u7528\u7A0B\u5E8F-\u00E9.exe").wstring();
				std::filesystem::path shortcutPath = shortcutFolder / (appName + L".url");

				DWORD writeError = ERROR_SUCCESS;
				bool writeResult = StartupShortcutManageAt(shortcutFolder.wstring(), appName, targetPath, &writeError);
				bool fileCreated = FileExists(shortcutPath.wstring());
				std::wstring readTarget = StartupShortcutReadTarget(shortcutPath.wstring());

				unsigned char byteOrderMark[2] = {};
				bool utf16Bom = false;
				{
					//Close the verification stream before the shortcut is removed
					std::ifstream shortcutStream(shortcutPath, std::ios::binary);
					shortcutStream.read((char*)byteOrderMark, 2);
					utf16Bom = byteOrderMark[0] == 0xFF && byteOrderMark[1] == 0xFE;
				}

				DWORD removeError = ERROR_SUCCESS;
				bool removeResult = StartupShortcutManageAt(shortcutFolder.wstring(), appName, targetPath, &removeError);
				bool fileRemoved = !FileExists(shortcutPath.wstring());
				bool shortcutPassed = writeResult && fileCreated && readTarget == targetPath && utf16Bom && removeResult && fileRemoved;
				addCase("startup_shortcut_unicode_target", shortcutPassed, L"path=" + shortcutPath.wstring() + L"; target=" + targetPath + L"; write_error=" + Utf8ToWide(Win32ErrorToString(writeError)) + L"; remove_error=" + Utf8ToWide(Win32ErrorToString(removeError)));

				//A missing folder must return the actual failure instead of reporting success
				std::filesystem::path missingShortcut = testRoot / L"missing-folder-\u4E2D\u6587" / L"RadeonTuner.url";
				DWORD missingError = ERROR_SUCCESS;
				bool missingResult = StartupShortcutWriteFile(missingShortcut.wstring(), targetPath, &missingError);
				addCase("startup_shortcut_failure_reported", !missingResult && missingError != ERROR_SUCCESS, L"path=" + missingShortcut.wstring() + L"; write_error=" + Utf8ToWide(Win32ErrorToString(missingError)));
			}
		}
		catch (std::exception const& exception)
		{
			addCase("unicode_test_exception", false, Utf8ToWide(exception.what()));
		}
		catch (...)
		{
			addCase("unicode_test_exception", false, L"Unknown exception");
		}

		report["all_passed"] = allPassed;
		report["finished_utc"] = UtcNowString();
		WriteUtf8File(std::filesystem::path(testDataPath).wstring() + L"\\unicode-path-tests.json", report.dump(2));

		//Remove only the test-owned temporary folder, the report remains in the test data folder
		std::error_code removeError;
		std::filesystem::remove_all(testRoot, removeError);
		return allPassed;
	}
}
