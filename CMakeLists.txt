cmake_minimum_required (VERSION 3.0)
project (ibus-eei)

cmake_policy(SET CMP0054 NEW)

add_compile_options(-Werror)
if (RELEASE EQUAL 1)
    message("Producing release build files")
    add_compile_options(-O3)
else()
    message("Producing debug build files")
endif()


set(LIBEXECDIR "${CMAKE_INSTALL_FULL_LIBEXECDIR}" CACHE STRING "where to install ibus-{engine,setup}-eei")
set(PKGDATADIR "${CMAKE_INSTALL_FULL_DATADIR}/${PROJECT_NAME}")
set(LOCALEDIR "${CMAKE_INSTALL_FULL_LOCALEDIR}")

set(CMAKE_EXE_LINKER_FLAGS  "${CMAKE_EXE_LINKER_FLAGS} -L ${LIBEXECDIR}")

add_definitions(
    -DPACKAGE_NAME="${PROJECT_NAME}"
    -DGETTEXT_PACKAGE="${PROJECT_NAME}"
    -DPACKAGE_VERSION="0.0.1"
    -DPACKAGE_BUGREPORT="https://github.com/Mindful/eei/issues"
    -DLOCALEDIR="${LOCALEDIR}"
    -DPKGDATADIR="${PKGDATADIR}"
    -DLIBEXECDIR="${LIBEXECDIR}"
)


add_subdirectory(src)

