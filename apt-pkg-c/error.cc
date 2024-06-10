#include "rust-apt/apt-pkg-c/error.h"
#include <apt-pkg/error.h>

#include "types.h"

Vec<AptError> get_all() noexcept {
	Vec<AptError> list;

	while (!_error->empty()) {
		std::string msg;
		bool type = _error->PopMessage(msg);
		list.push_back(AptError{type, msg});
	}
	return list;
}
