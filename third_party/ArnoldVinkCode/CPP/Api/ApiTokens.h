#pragma once
#include <string>

namespace ArnoldVinkCode
{
	//GitHub API access token is optional.
	//Leave empty to use public, unauthenticated GitHub API requests.
	//Never commit a real credential to this file.
	struct ApiTokens
	{
		static inline std::string GitHub = "";
	};
}
