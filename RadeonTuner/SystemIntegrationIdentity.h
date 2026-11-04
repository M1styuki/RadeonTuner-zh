#pragma once
#include <string>

//Identity used only by this Simplified Chinese build for Windows startup/start-menu shortcuts
//and the Explorer background context menu. It is deliberately different from the original
//RadeonTuner entries so this build never reads, replaces or removes the original integration.
namespace RadeonTunerSystemIntegration
{
	inline constexpr wchar_t ApplicationName[] = L"RadeonTuner 简体中文版";
	inline constexpr wchar_t ContextMenuKey[] = L"RadeonTunerZhCN";

	inline std::wstring QuoteExecutablePath(const std::wstring& executablePath)
	{
		return L"\"" + executablePath + L"\"";
	}

	inline std::wstring ContextMenuCommand(const std::wstring& executablePath)
	{
		//The registry command must keep a path containing spaces/Chinese together as one token.
		return QuoteExecutablePath(executablePath);
	}
}
