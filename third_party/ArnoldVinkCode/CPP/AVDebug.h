#pragma once
#include <windows.h>
#include <iostream>
#include <sstream>
#include <fstream>
#include <string>
#include "AVString.h"
#define AVDebugWriteLine(message) AVDebugWriteLineInternal() << message
inline bool AVDebugWriteLineLogFile = false;
inline bool AVDebugWriteLineLogFileEnabled = false;
inline std::wstring AVDebugWriteLineLogFilePath = L"";

//Usage example: AVDebugWriteLine("Hello " << "World " << 1);
//Usage example: AVDebugWriteLineInternal() << "Hello " << "World " << 1;
//Description: Show string in both console and debug output
struct AVDebugWriteLineInternal
{
	std::wostringstream stringstream;

	template <typename T>
	AVDebugWriteLineInternal& operator << (const T& value)
	{
		stringstream << value;
		return *this;
	}

	AVDebugWriteLineInternal& operator << (const std::string& value)
	{
		stringstream << ArnoldVinkCode::string_to_wstring_utf8(value);
		return *this;
	}

	~AVDebugWriteLineInternal()
	{
		stringstream << "\n";
		OutputDebugStringW(stringstream.str().c_str());
		std::wcout << stringstream.str().c_str() << std::flush;
		if (AVDebugWriteLineLogFile || AVDebugWriteLineLogFileEnabled)
		{
			std::wofstream logstream(AVDebugWriteLineLogFilePath.empty() ? L"Debug.log" : AVDebugWriteLineLogFilePath.c_str(), std::ios::app);
			logstream << stringstream.str().c_str();
		}
	}
};