#pragma once
#include <chrono>
#include <winrt/Windows.Foundation.h>
#include <winrt/Windows.UI.Core.h>

namespace ArnoldVinkCode
{
	//Awaitable delay used by the UI code before the next profile load.
	//When a dispatcher is provided, continue on that UI thread.
	inline winrt::Windows::Foundation::IAsyncAction AsyncTaskDelay(int milliseconds, winrt::Windows::UI::Core::CoreDispatcher dispatcher = nullptr)
	{
		if (milliseconds < 0)
		{
			milliseconds = 0;
		}
		co_await winrt::resume_after(std::chrono::milliseconds(milliseconds));
		if (dispatcher != nullptr)
		{
			co_await winrt::resume_foreground(dispatcher);
		}
	}
}
