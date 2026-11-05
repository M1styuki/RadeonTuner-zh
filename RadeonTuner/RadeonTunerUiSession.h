#pragma once

namespace winrt::RadeonTuner::implementation
{
	struct MainPage;
}

//UI session helpers implemented in MainPage.cpp. The snapshot stores plain
//control values only, never a XAML object or a page pointer, so it survives the
//page and DesktopWindowXamlSource being released.
namespace RadeonTunerUiSession
{
	void Capture(winrt::RadeonTuner::implementation::MainPage& page);
	void PrepareForRelease(winrt::RadeonTuner::implementation::MainPage& page);
	void Restore(winrt::RadeonTuner::implementation::MainPage& page);
	bool HasSnapshot();
	void ClearSnapshot();
}
