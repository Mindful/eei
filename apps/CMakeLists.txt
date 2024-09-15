add_executable(ibus-engine-eei
        main.c
        engine.c
)
target_link_libraries(ibus-engine-eei
    ${IBUS_LIBRARIES} predict
)
target_include_directories(ibus-engine-eei PUBLIC ${IBUS_INCLUDE_DIRS} ../include)

configure_file(eei.xml.in eei.xml)

install(TARGETS ibus-engine-eei DESTINATION ${LIBEXECDIR})
install(FILES ${CMAKE_CURRENT_BINARY_DIR}/eei.xml DESTINATION "${PKGDATADIR}/ibus/component")
