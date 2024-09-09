cmake_minimum_required (VERSION 3.22)
project (ibus-eei)

cmake_policy(SET CMP0054 NEW)

# Include install directory variables as defined by the GNU Coding Standards.
include(GNUInstallDirs)

add_compile_options(-Wall -Wextra)

set(LIBEXECDIR "${CMAKE_INSTALL_FULL_LIBEXECDIR}")
set(PKGDATADIR "${CMAKE_INSTALL_FULL_DATADIR}")
set(LOCALEDIR "${CMAKE_INSTALL_FULL_LOCALEDIR}")

add_definitions(
    -DPACKAGE_NAME="${PROJECT_NAME}"
    -DGETTEXT_PACKAGE="${PROJECT_NAME}"
    -DPACKAGE_VERSION="0.0.1"
    -DPACKAGE_BUGREPORT="https://github.com/Mindful/eei/issues"
    -DLOCALEDIR="${LOCALEDIR}"
    -DPKGDATADIR="${PKGDATADIR}"
    -DLIBEXECDIR="${LIBEXECDIR}"
)

### Require out-of-source builds
file(TO_CMAKE_PATH "${PROJECT_BINARY_DIR}/CMakeLists.txt" LOC_PATH)
if(EXISTS "${LOC_PATH}")
    message(FATAL_ERROR "You cannot build in a source directory (or any directory with a CMakeLists.txt file). Please make a build subdirectory. Feel free to remove CMakeCache.txt and CMakeFiles.")
endif()

# FetchContent added in CMake 3.11, downloads during the configure step
# FetchContent_MakeAvailable was added in CMake 3.14; simpler usage
include(FetchContent)

find_package(PkgConfig)
pkg_check_modules(IBUS REQUIRED ibus-1.0)

# Corrosion tools
FetchContent_Declare(
    Corrosion
    GIT_REPOSITORY https://github.com/corrosion-rs/corrosion.git
    GIT_TAG v0.5 # Optionally specify a commit hash, version tag or branch here
)
FetchContent_MakeAvailable(Corrosion)
# Adds Corrosion

# The compiled library code here
add_subdirectory(src)

# The executable code here
add_subdirectory(apps)

# Testing only available if this is the main app
# Emergency override MODERN_CMAKE_BUILD_TESTING provided as well
if((CMAKE_PROJECT_NAME STREQUAL PROJECT_NAME)
   AND BUILD_TESTING)
  include(CTest)
  add_subdirectory(tests)
endif()
