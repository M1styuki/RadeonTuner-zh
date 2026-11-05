#include "../RadeonTuner/FanInputValue.h"

#include <cassert>
#include <iostream>

int main()
{
	using RadeonTuner::FanInputValue::TryParse;

	int value = -1;
	assert(TryParse(L" 45 ", 20, 100, 5, value) && value == 45);
	assert(TryParse(L"42", 20, 100, 5, value) && value == 40);
	assert(TryParse(L"1000", 20, 100, 5, value) && value == 100);
	assert(TryParse(L"-5", 20, 100, 5, value) && value == 20);
	assert(TryParse(L"+65", 0, 100, 1, value) && value == 65);
	assert(TryParse(L"0", 20, 100, 1, value) && value == 20);

	value = 77;
	assert(!TryParse(L"", 20, 100, 1, value) && value == 77);
	assert(!TryParse(L"   ", 20, 100, 1, value) && value == 77);
	assert(!TryParse(L"4x", 20, 100, 1, value) && value == 77);
	assert(!TryParse(L"12.5", 20, 100, 1, value) && value == 77);
	assert(!TryParse(L"-", 20, 100, 1, value) && value == 77);
	assert(!TryParse(L"999999999999999999999999", 20, 100, 1, value) && value == 77);

	std::cout << "Fan input parse/clamp/snap checks passed\n";
	return 0;
}
