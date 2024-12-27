# dynamips - build system
# Control variables:
#  - DYNAMIPS_RENAME : rename executable to dynamips (auto;stable;unstable;<empty>)
#  - DYNAMIPS_CODE   : set default code (stable;unstable;both)
#  - DYNAMIPS_ARCH   : build target architecture (amd64;x86;nojit)
#  - ANY_COMPILER    : skip compiler check (assumes gcc)
#  - CMAKE_INSTALL_PREFIX : where to install

cmake_minimum_required ( VERSION 3.15 )
foreach ( _dir
   "/usr/sfw" # search in "Sun FreeWare" (SunOS)
   "/opt/csw" # search in "OpenCSW" (SunOS)
   )
   if ( IS_ABSOLUTE "${_dir}" AND IS_DIRECTORY "${_dir}" )
      list ( APPEND CMAKE_PREFIX_PATH "${_dir}" )
   endif ()
endforeach ()
if ( CMAKE_PREFIX_PATH )
   message ( STATUS "CMAKE_PREFIX_PATH=${CMAKE_PREFIX_PATH}" )
endif ( CMAKE_PREFIX_PATH )
list ( APPEND CMAKE_MODULE_PATH "${CMAKE_CURRENT_SOURCE_DIR}/cmake" )
message ( STATUS "CMAKE_MODULE_PATH=${CMAKE_MODULE_PATH}" )

if( NOT CMAKE_BUILD_TYPE ) # make sure the build type is set
   set( CMAKE_BUILD_TYPE "RelWithDebInfo" )
   message( WARNING "CMAKE_BUILD_TYPE is not set, defaulting to ${CMAKE_BUILD_TYPE}" )
endif()

project ( dynamips 
   VERSION 0.2.23
   LANGUAGES C
)
set ( DYNAMIPS_VERSION_TRAIN "${PROJECT_VERSION}" )
set ( DYNAMIPS_VERSION_SUB )

include ( utils )
include ( dependencies )
include ( configure )

if( APPLE OR (CMAKE_SYSTEM_NAME MATCHES "FreeBSD") )
    include_directories(include /usr/local/include/)
    include_directories(include /opt/local/include/)
endif()

# Enable testing
set( CTEST_TEST_TIMEOUT 0 ) # disable timeout
include(CTest)

# import rust crates
set ( _extra_crates )
if ( BUILD_UDP_RECV )
   set( _extra_crates ${_extra_crates} udp_recv )
endif()
if ( BUILD_UDP_SEND )
   set( _extra_crates ${_extra_crates} udp_send )
endif()
corrosion_import_crate(
   MANIFEST_PATH "${CMAKE_SOURCE_DIR}/Cargo.toml"
   PROFILE "cmake-${CMAKE_BUILD_TYPE}"
   CRATES
      dynamips-c-stable # dynamips_c_stable
      dynamips-c-unstable # dynamips_c_unstable
      ${_extra_crates}
   FEATURES ${DYNAMIPS_FEATURES}
)
if ( _extra_crates )
corrosion_install( TARGETS ${_extra_crates} )
endif()
# generate dynamips_c.h
add_library( dynamips_c_h INTERFACE ) # FIXME workaround for corrosion_experimental_cbindgen(MANIFEST_DIRECTORY)
set_target_properties( dynamips_c_h PROPERTIES
   INTERFACE_COR_PACKAGE_MANIFEST_PATH "${CMAKE_SOURCE_DIR}/dynamips-c/Cargo.toml"
   INTERFACE_COR_CARGO_PACKAGE_NAME "dynamips-c"
)
corrosion_experimental_cbindgen( TARGET dynamips_c_h HEADER_NAME "dynamips_c.h" )

string( REPLACE ";" "," _features_commas "${DYNAMIPS_FEATURES}" )
add_custom_target( cargo_fmt COMMAND cargo fmt -v WORKING_DIRECTORY "${CMAKE_SOURCE_DIR}" USES_TERMINAL )
add_custom_target( cargo_clippy COMMAND cargo clippy "--profile=cmake-${CMAKE_BUILD_TYPE}" --features=${_features_commas} --target-dir "${CMAKE_BINARY_DIR}/cargo/build" -v WORKING_DIRECTORY "${CMAKE_SOURCE_DIR}" USES_TERMINAL )
add_custom_target( cargo_test COMMAND cargo test "--profile=cmake-${CMAKE_BUILD_TYPE}" --features=${_features_commas} --target-dir "${CMAKE_BINARY_DIR}/cargo/build" -v -- --test-threads 1 WORKING_DIRECTORY "${CMAKE_SOURCE_DIR}" USES_TERMINAL )
add_custom_target( cargo_doc COMMAND cargo doc --features=${_features_commas} --target-dir "${CMAKE_BINARY_DIR}/cargo/build" -v WORKING_DIRECTORY "${CMAKE_SOURCE_DIR}" USES_TERMINAL )

add_test( NAME test_cargo_fmt COMMAND cargo fmt --check -v WORKING_DIRECTORY "${CMAKE_SOURCE_DIR}" )
set_tests_properties( test_cargo_fmt PROPERTIES TIMEOUT 0 )
add_test( NAME test_cargo_clippy COMMAND cargo clippy "--profile=cmake-${CMAKE_BUILD_TYPE}" --features=${_features_commas} --target-dir "${CMAKE_BINARY_DIR}/cargo/build" -v -- -D warnings WORKING_DIRECTORY "${CMAKE_SOURCE_DIR}" )
set_tests_properties( test_cargo_clippy PROPERTIES TIMEOUT 0 )
add_test( NAME test_cargo_test COMMAND cargo test "--profile=cmake-${CMAKE_BUILD_TYPE}" --features=${_features_commas} --target-dir "${CMAKE_BINARY_DIR}/cargo/build" -v -- --test-threads 1 WORKING_DIRECTORY "${CMAKE_SOURCE_DIR}" )
set_tests_properties( test_cargo_test PROPERTIES TIMEOUT 0 )

add_subdirectory ( man )
add_subdirectory ( common )
add_subdirectory ( stable )
add_subdirectory ( unstable )

install_docs (
   "ChangeLog"
   "COPYING"
   "MAINTAINERS"
   "README.md"
   "README.hypervisor"
   "RELEASE-NOTES"
   "TODO"
   )

# uninstall target - leaves behind any directory created during install
set ( _templatefile "${CMAKE_SOURCE_DIR}/cmake/cmake_uninstall.cmake.in" )
set ( _scriptfile "${CMAKE_BINARY_DIR}/cmake_uninstall.cmake" )
configure_file ( "${_templatefile}" "${_scriptfile}" IMMEDIATE @ONLY )
add_custom_target (
   uninstall
   COMMAND ${CMAKE_COMMAND} -P "${_scriptfile}"
   COMMENT "Uninstall files listed in install_manifest.txt"
   )

print_summary ()

# Enable packaging
include(CPack)

enable_testing()
