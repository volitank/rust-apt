#pragma once
#include <apt-pkg/cachefile.h>

using PkgCacheFile = pkgCacheFile;
// DepCache is owned by the PkgCacheFile.
// Needs to be * to prevent CXX from deleting it.
using PkgDepCache = pkgDepCache*;
using IndexFile = pkgIndexFile*;
using PkgActionGroup = pkgDepCache::ActionGroup;
using PkgIterator = pkgCache::PkgIterator;
using VerIterator = pkgCache::VerIterator;
using PrvIterator = pkgCache::PrvIterator;
using DepIterator = pkgCache::DepIterator;
using VerFileIterator = pkgCache::VerFileIterator;
using DescFileIterator = pkgCache::DescFileIterator;
using PkgFileIterator = pkgCache::PkgFileIterator;
