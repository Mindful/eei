find_package(PkgConfig)
pkg_check_modules(IBUS REQUIRED ibus-1.0)

# Rust requires PThreads
set(THREADS_PREFER_PTHREAD_FLAG ON)
find_package(Threads REQUIRED)


if (RELEASE EQUAL 1)
    find_library(PREDICT src/predict/target/release/libpredict.a)
else()
    find_library(PREDICT src/predict/target/debug/libpredict.a)
endif()

add_executable(ibus-engine-eei
        main.c
        engine.c
)
target_link_libraries(ibus-engine-eei
    ${IBUS_LIBRARIES} ${PREDICT} Threads::Threads dl m # pthreads, dl, m required by rust
)
target_include_directories(ibus-engine-eei PUBLIC ${IBUS_INCLUDE_DIRS})

configure_file(eei.xml.in eei.xml)

install(TARGETS ibus-engine-eei DESTINATION ${LIBEXECDIR})
install(FILES ${CMAKE_CURRENT_BINARY_DIR}/eei.xml DESTINATION "${CMAKE_INSTALL_FULL_DATADIR}/ibus/component")
