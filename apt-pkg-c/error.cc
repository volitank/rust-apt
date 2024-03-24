#include "rust-apt/apt-pkg-c/error.h"
#include <apt-pkg/error.h>

rust::Vec<AptError> get_all() noexcept {
	rust::Vec<AptError> list;

	while (!_error->empty()) {
		std::string msg;
		bool type = _error->PopMessage(msg);
		list.push_back(AptError{type, msg});
	}
	return list;
}
