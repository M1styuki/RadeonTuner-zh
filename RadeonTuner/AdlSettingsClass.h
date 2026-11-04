#pragma once
#include "pch.h"

enum class AdlSettingGet
{
	Current,
	Default
};

struct AdlSettingVersion
{
	int Version;
};

struct AdlSettingInt
{
	std::optional<int> Current;
	std::optional<int> Default;
	std::optional<int> Minimum;
	std::optional<int> Maximum;
	std::optional<int> Step;
	bool Support;

	std::optional<int> Get(AdlSettingGet settingType)
	{
		return settingType == AdlSettingGet::Current ? Current : Default;
	}

	std::optional<int> Get(AdlSettingGet settingType, bool fallbackDefault)
	{
		std::optional<int> settingValue = settingType == AdlSettingGet::Current ? Current : Default;
		if (!settingValue.has_value() && fallbackDefault)
		{
			settingValue = settingType == AdlSettingGet::Current ? Default : Current;
		}
		return settingValue;
	}
};

struct AdlSettingFloat
{
	std::optional<float> Current;
	std::optional<float> Default;
	std::optional<float> Minimum;
	std::optional<float> Maximum;
	std::optional<float> Step;
	bool Support;

	std::optional<float> Get(AdlSettingGet settingType)
	{
		return settingType == AdlSettingGet::Current ? Current : Default;
	}

	std::optional<float> Get(AdlSettingGet settingType, bool fallbackDefault)
	{
		std::optional<float> settingValue = settingType == AdlSettingGet::Current ? Current : Default;
		if (!settingValue.has_value() && fallbackDefault)
		{
			settingValue = settingType == AdlSettingGet::Current ? Default : Current;
		}
		return settingValue;
	}
};

struct AdlSettingString
{
	std::optional<std::wstring> Current;
	std::optional<std::wstring> Default;
	bool Support;

	std::optional<std::wstring> Get(AdlSettingGet settingType)
	{
		return settingType == AdlSettingGet::Current ? Current : Default;
	}

	std::optional<std::wstring> Get(AdlSettingGet settingType, bool fallbackDefault)
	{
		std::optional<std::wstring> settingValue = settingType == AdlSettingGet::Current ? Current : Default;
		if (!settingValue.has_value() && fallbackDefault)
		{
			settingValue = settingType == AdlSettingGet::Current ? Default : Current;
		}
		return settingValue;
	}
};