#pragma once
#include "pch.h"
#include <winsvc.h>
#pragma comment(lib, "Advapi32.lib")
#include "MainPage.h"

namespace winrt::RadeonTuner::implementation
{
	void MainPage::ToggleRadeonServices(bool disableServices)
	{
		try
		{
			//Only touch the optional AMD background services used by the driver features.
			const wchar_t* serviceNames[] =
			{
				L"AMDRSServ",
				L"AMD External Events Utility",
				L"AMD Crash Defender Service"
			};

			SC_HANDLE serviceManager = OpenSCManagerW(NULL, NULL, SC_MANAGER_CONNECT);
			if (serviceManager == NULL)
			{
				AVDebugWriteLine(L"Failed opening the service manager: " << GetLastError());
				return;
			}

			for (const wchar_t* serviceName : serviceNames)
			{
				DWORD access = SERVICE_CHANGE_CONFIG | (disableServices ? SERVICE_STOP : SERVICE_START);
				SC_HANDLE service = OpenServiceW(serviceManager, serviceName, access);
				if (service == NULL)
				{
					continue;
				}

				if (disableServices)
				{
					//Ask the service to stop, then prevent it from starting with Windows again.
					SERVICE_STATUS serviceStatus{};
					ControlService(service, SERVICE_CONTROL_STOP, &serviceStatus);
					ChangeServiceConfigW(service, SERVICE_NO_CHANGE, SERVICE_DISABLED, SERVICE_NO_CHANGE, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
				}
				else
				{
					//Restore automatic startup and start the service again.
					ChangeServiceConfigW(service, SERVICE_NO_CHANGE, SERVICE_AUTO_START, SERVICE_NO_CHANGE, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
					StartServiceW(service, 0, NULL);
				}
				CloseServiceHandle(service);
			}

			CloseServiceHandle(serviceManager);
		}
		catch (...)
		{
			AVDebugWriteLine("Failed toggling Radeon services.");
		}
	}

	void MainPage::button_DisableRadeonServices_Click(IInspectable const& sender, RoutedEventArgs const& e)
	{
		ToggleRadeonServices(true);
	}

	void MainPage::button_EnableRadeonServices_Click(IInspectable const& sender, RoutedEventArgs const& e)
	{
		ToggleRadeonServices(false);
	}
}
