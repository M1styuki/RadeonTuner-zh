#pragma once

#include <string>

//Backend loop entry points owned by App, implemented in MainPage.cpp so the
//non-inline ADL helper definitions stay in a single translation unit.
namespace RadeonTunerBackend
{
	struct HeadlessOptions
	{
		bool InitializeAdl = true;
		bool LoadProfiles = true;
		bool SelectDefaults = true;
		bool WriteDefaults = false;
		bool SyncApplications = false;
	};

	//Native-thread headless initialization. It never constructs MainPage or XAML.
	//Returns an empty string on success, or the original ADL/error text on failure.
	std::wstring InitializeHeadless(HeadlessOptions const& options);

	//Reload the same existing profile JSON files and settings cache used by the UI.
	bool ReloadProfiles();
	void ReloadSettings();

	//Full current model/selection plus plain UI snapshot, used by host-worker IPC.
	std::string CaptureStateJson();
	bool ApplyStateJson(std::string const& json);
	bool ApplyRuntimeStateJson(std::string const& json);
	std::string MergeRuntimeStateJson(std::string const& draftJson, std::string const& runtimeJson);

	//Pause blocks until any in-flight profile batch has completed.
	void PauseProfileLoop();
	void ResumeProfileLoop();
	void MetricsLoop();
	void ProfileLoop();
}
