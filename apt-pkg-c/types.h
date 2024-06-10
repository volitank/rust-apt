#pragma once
#include <apt-pkg/indexfile.h>
#include <memory>
#include "rust/cxx.h"

using namespace rust;

template <typename T>
using UniquePtr = std::unique_ptr<T>;

// Forward declarations for progress.rs
struct ItemDesc;
struct PkgAcquire;
struct AcqWorker;
