#pragma once

#include <Windows.h>
#include <cstdint>
#include <string>

//Small length-prefixed framing for the inherited anonymous pipes. No named
//pipe or session file is created; the host owns both pipe lifetimes.
namespace RadeonTunerIpc
{
	inline bool ReadExact(HANDLE pipe, void* buffer, DWORD bytes)
	{
		DWORD total = 0;
		while (total < bytes)
		{
			DWORD read = 0;
			if (!ReadFile(pipe, static_cast<BYTE*>(buffer) + total, bytes - total, &read, NULL) || read == 0)
			{
				return false;
			}
			total += read;
		}
		return true;
	}

	inline bool ReadFrame(HANDLE pipe, std::string& payload)
	{
		uint32_t length = 0;
		if (!ReadExact(pipe, &length, sizeof(length)))
		{
			return false;
		}
		constexpr uint32_t maximumLength = 8 * 1024 * 1024;
		if (length == 0 || length > maximumLength)
		{
			return false;
		}
		payload.assign(length, '\0');
		return ReadExact(pipe, payload.data(), length);
	}

	inline bool WriteFrame(HANDLE pipe, std::string const& payload)
	{
		constexpr uint32_t maximumLength = 8 * 1024 * 1024;
		if (payload.empty() || payload.size() > maximumLength)
		{
			return false;
		}
		uint32_t length = static_cast<uint32_t>(payload.size());
		DWORD written = 0;
		if (!WriteFile(pipe, &length, sizeof(length), &written, NULL) || written != sizeof(length))
		{
			return false;
		}
		if (!WriteFile(pipe, payload.data(), length, &written, NULL) || written != length)
		{
			return false;
		}
		FlushFileBuffers(pipe);
		return true;
	}
}
