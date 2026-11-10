#pragma once
#include "pch.h"
#include <winsvc.h>

//Minimal Windows Service Control Manager helpers used by the manual
//Radeon service buttons. No service is changed unless the user clicks.
inline bool WindowsServiceDisableEnable(std::wstring const& serviceName, bool disableService)
{
	SC_HANDLE manager = OpenSCManagerW(NULL, NULL, SC_MANAGER_CONNECT);
	if (manager == NULL) { return false; }
	SC_HANDLE service = OpenServiceW(manager, serviceName.c_str(), SERVICE_CHANGE_CONFIG | SERVICE_QUERY_STATUS);
	if (service == NULL) { CloseServiceHandle(manager); return false; }
	BOOL changed = ChangeServiceConfigW(service, SERVICE_NO_CHANGE, disableService ? SERVICE_DISABLED : SERVICE_DEMAND_START, SERVICE_NO_CHANGE, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
	CloseServiceHandle(service);
	CloseServiceHandle(manager);
	return changed != FALSE;
}

inline bool WindowsServiceStartStop(std::wstring const& serviceName, bool startService)
{
	SC_HANDLE manager = OpenSCManagerW(NULL, NULL, SC_MANAGER_CONNECT);
	if (manager == NULL) { return false; }
	SC_HANDLE service = OpenServiceW(manager, serviceName.c_str(), SERVICE_START | SERVICE_STOP | SERVICE_QUERY_STATUS);
	if (service == NULL) { CloseServiceHandle(manager); return false; }
	bool result = false;
	if (startService)
	{
		result = StartServiceW(service, 0, NULL) != FALSE || GetLastError() == ERROR_SERVICE_ALREADY_RUNNING;
	}
	else
	{
		SERVICE_STATUS status{};
		result = ControlService(service, SERVICE_CONTROL_STOP, &status) != FALSE || GetLastError() == ERROR_SERVICE_NOT_ACTIVE;
	}
	CloseServiceHandle(service);
	CloseServiceHandle(manager);
	return result;
}
