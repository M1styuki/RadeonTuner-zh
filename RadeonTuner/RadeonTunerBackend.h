#pragma once

//Backend loop entry points owned by App, implemented in MainPage.cpp so the
//non-inline ADL helper definitions stay in a single translation unit.
namespace RadeonTunerBackend
{
	void MetricsLoop();
	void ProfileLoop();
}
