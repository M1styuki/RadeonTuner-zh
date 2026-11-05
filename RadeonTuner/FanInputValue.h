#pragma once
#include <climits>
#include <cmath>
#include <string>

namespace RadeonTuner::FanInputValue
{
	//Parse a complete integer and apply the adapter slider's actual range and step.
	//A blank, fractional, partially numeric or out-of-range string is rejected without
	//touching the caller's value. The caller only assigns the adapter when this returns true.
	inline bool TryParse(std::wstring const& text, double minimum, double maximum, double step, int& value)
	{
		size_t first = text.find_first_not_of(L" \t\r\n");
		if (first == std::wstring::npos) { return false; }
		size_t last = text.find_last_not_of(L" \t\r\n");
		std::wstring trimmed = text.substr(first, last - first + 1);

		size_t index = 0;
		bool negative = false;
		if (trimmed[index] == L'+' || trimmed[index] == L'-')
		{
			negative = trimmed[index] == L'-';
			index++;
		}
		if (index >= trimmed.size()) { return false; }

		long long parsed = 0;
		for (; index < trimmed.size(); index++)
		{
			wchar_t character = trimmed[index];
			if (character < L'0' || character > L'9') { return false; }
			int digit = character - L'0';
			if (parsed > (LLONG_MAX - digit) / 10) { return false; }
			parsed = parsed * 10 + digit;
		}
		if (negative) { parsed = -parsed; }

		long long minimumInteger = (long long)std::ceil(minimum);
		long long maximumInteger = (long long)std::floor(maximum);
		if (minimumInteger > maximumInteger) { return false; }
		if (parsed < minimumInteger) { parsed = minimumInteger; }
		if (parsed > maximumInteger) { parsed = maximumInteger; }

		if (step > 0.0)
		{
			long long snapped = (long long)std::llround(minimum + std::round((parsed - minimum) / step) * step);
			if (snapped < minimumInteger) { snapped = minimumInteger; }
			if (snapped > maximumInteger) { snapped = maximumInteger; }
			parsed = snapped;
		}

		value = (int)parsed;
		return true;
	}
}
