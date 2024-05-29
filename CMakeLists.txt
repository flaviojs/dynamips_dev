# dynamips - build system
# Control variables:
#  - DYNAMIPS_RENAME : rename executable to dynamips (auto;stable;unstable;<empty>)
#  - DYNAMIPS_CODE   : set default code (stable;unstable;both)
#  - DYNAMIPS_ARCH   : build target architecture (amd64;x86;nojit)
#  - ANY_COMPILER    : skip compiler check (assumes gcc)
#  - CMAKE_INSTALL_PREFIX : where to install

set ( CMAKE_LEGACY_CYGWIN_WIN32 0 ) # Remove when CMake >= 2.8.4 is required
cmake_minimum_required ( VERSION 2.8 )
cmake_policy ( VERSION 2.8 )
cmake_policy ( SET CMP0017 NEW ) # Prefer files from the CMake module directory when including from there.
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

project ( dynamips C )
set ( DYNAMIPS_VERSION_TRAIN 0.2.23 )
set ( DYNAMIPS_VERSION_SUB )

include ( utils )
include ( dependencies )
include ( configure )

if( APPLE OR (CMAKE_SYSTEM_NAME MATCHES "FreeBSD") )
    include_directories(include /usr/local/include/)
    include_directories(include /opt/local/include/)
endif()

# Rust code
include ( FetchContent )
FetchContent_Declare (
   Corrosion
   GIT_REPOSITORY https://github.com/corrosion-rs/corrosion.git
   GIT_TAG v0.5
)
FetchContent_MakeAvailable ( Corrosion )
corrosion_import_crate (
  MANIFEST_PATH "${CMAKE_SOURCE_DIR}/rust/Cargo.toml"
  PROFILE release-with-debug
  CRATES
    dynamips_c
    dynamips_c_unstable
    nvram_export
    udp_recv
    udp_send
)
function ( _set_features _crate )
   set ( _features )
   foreach ( _feature IN ITEMS ${ARGV} )
      if( ${_feature} )
         list ( APPEND _features "${_feature}" )
      endif()
   endforeach ()
   corrosion_set_features ( "${_crate}" FEATURES ${_features} )
endfunction ()
_set_features( dynamips_c ENABLE_LINUX_ETH ENABLE_GEN_ETH ENABLE_IPV6 )
_set_features( dynamips_c_unstable ENABLE_LINUX_ETH ENABLE_GEN_ETH ENABLE_IPV6 )
_set_features( udp_recv ENABLE_IPV6 )
_set_features( udp_send ENABLE_IPV6 )
corrosion_experimental_cbindgen( MANIFEST_PATH "${CMAKE_SOURCE_DIR}/rust/dynamips_c/Cargo.toml" TARGET dynamips_c HEADER_NAME "rust_dynamips_c.h" )
target_include_directories ( dynamips_c_unstable INTERFACE "${CMAKE_CURRENT_BINARY_DIR}/corrosion_generated/cbindgen/dynamips_c/include" ) 
add_dependencies ( dynamips_c_unstable "_corrosion_cbindgen_dynamips_c_bindings" )

add_custom_target ( cargo_fmt COMMAND cargo fmt -v WORKING_DIRECTORY "${CMAKE_SOURCE_DIR}/rust" USES_TERMINAL )
add_custom_target ( cargo_clippy COMMAND cargo clippy --target-dir "${CMAKE_BINARY_DIR}/cargo/build" -v WORKING_DIRECTORY "${CMAKE_SOURCE_DIR}/rust" USES_TERMINAL )
add_custom_target ( cargo_test COMMAND cargo test --target-dir "${CMAKE_BINARY_DIR}/cargo/build" -v -- --test-threads 1 WORKING_DIRECTORY "${CMAKE_SOURCE_DIR}/rust" USES_TERMINAL )
add_custom_target ( cargo_doc COMMAND cargo doc --target-dir "${CMAKE_BINARY_DIR}/cargo/build" -v WORKING_DIRECTORY "${CMAKE_SOURCE_DIR}/rust" USES_TERMINAL )

enable_testing()
add_test( NAME test_cargo_fmt COMMAND cargo fmt --check -v WORKING_DIRECTORY "${CMAKE_SOURCE_DIR}/rust" )
add_test( NAME test_cargo_clippy COMMAND cargo clippy --target-dir "${CMAKE_BINARY_DIR}/cargo/build" -v -- -D warnings WORKING_DIRECTORY "${CMAKE_SOURCE_DIR}/rust" )
add_test( NAME test_cargo_test COMMAND cargo test --target-dir "${CMAKE_BINARY_DIR}/cargo/build" -v -- --test-threads 1 WORKING_DIRECTORY "${CMAKE_SOURCE_DIR}/rust" )

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
