#pragma once
#include <string>
#include "UiValidationCommon.h"
#include "SystemIntegrationIdentity.h"

//Pure generated-value checks only: this never reads/writes the real startup folder or registry.
namespace RadeonTunerUiValidation::SystemIntegrationTests
{
	inline bool Run(std::wstring const& testDataPath)
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
		};

		const std::wstring expectedName = L"RadeonTuner \u7B80\u4F53\u4E2D\u6587\u7248";
		const std::wstring expectedKey = L"RadeonTunerZhCN";
		addCase("independent_application_name",
			RadeonTunerSystemIntegration::ApplicationName == expectedName && expectedName != L"RadeonTuner",
			L"name=" + std::wstring(RadeonTunerSystemIntegration::ApplicationName) + L"; shortcut=" + expectedName + L".url");
		addCase("independent_context_menu_key",
			RadeonTunerSystemIntegration::ContextMenuKey == expectedKey && expectedKey != L"RadeonTuner",
			L"key=" + std::wstring(RadeonTunerSystemIntegration::ContextMenuKey));

		std::wstring unicodePath = L"C:\\Users\\\u6D4B\u8BD5 \u7528\u6237\\AppData\\Local\\RadeonTuner-zh-CN\\App\\RadeonTuner.exe";
		std::wstring command = RadeonTunerSystemIntegration::ContextMenuCommand(unicodePath);
		bool quoted = command.size() == unicodePath.size() + 2 && command.front() == L'"' && command.back() == L'"' && command.substr(1, unicodePath.size()) == unicodePath;
		addCase("quoted_context_menu_command", quoted, L"command=" + command);

		std::wstring plainPath = L"C:\\RadeonTuner\\RadeonTuner.exe";
		std::wstring plainCommand = RadeonTunerSystemIntegration::ContextMenuCommand(plainPath);
		addCase("quoted_command_without_spaces", plainCommand == L"\"" + plainPath + L"\"", L"command=" + plainCommand);

		report["all_passed"] = allPassed;
		report["finished_utc"] = UtcNowString();
		report["note"] = "Pure generated values only; no real shortcut or registry entry was read, created or removed.";
		WriteUtf8File(testDataPath + L"\\system-integration-tests.json", report.dump(2));
		return allPassed;
	}
}
