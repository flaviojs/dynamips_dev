# dynamips - build system
# Control variables:
#  - DYNAMIPS_RENAME : rename executable to dynamips (auto;stable;unstable;<empty>)
#  - DYNAMIPS_CODE   : set default code (stable;unstable;both)
#  - DYNAMIPS_ARCH   : build target architecture (amd64;x86;nojit)
#  - ANY_COMPILER    : skip compiler check (assumes gcc)
#  - CMAKE_INSTALL_PREFIX : where to install

cmake_minimum_required( VERSION 3.1 )
cmake_policy( VERSION 3.1 )
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
include( FetchContent )
FetchContent_Declare(
   Corrosion
   GIT_REPOSITORY https://github.com/corrosion-rs/corrosion.git
   GIT_TAG v0.5
)
FetchContent_MakeAvailable( Corrosion )
function( _get_feature_list _var )
   set( _features )
   foreach( _feature IN ITEMS ${ARGN} )
      if( "${_feature}" STREQUAL "DYNAMIPS_ARCH" )
         # string option
         if( DYNAMIPS_ARCH STREQUAL "amd64" )
            list( APPEND _features "USE_MIPS64_AMD64_TRANS" "USE_PPC32_AMD64_TRANS" )
         elseif( DYNAMIPS_ARCH STREQUAL "x86" )
            list( APPEND _features "USE_MIPS64_X86_TRANS" "USE_PPC32_X86_TRANS" )
         elseif( DYNAMIPS_ARCH STREQUAL "nojit" )
            list( APPEND _features "USE_MIPS64_NOJIT_TRANS" "USE_PPC32_NOJIT_TRANS" )
         else()
            message( FATAL_ERROR "TODO features for DYNAMIPS_ARCH=${DYNAMIPS_ARCH}" )
         endif()
      elseif( ${_feature} )
         # boolean options
         list( APPEND _features "${_feature}" )
      endif()
   endforeach()
   set( ${_var} ${_features} PARENT_SCOPE )
endfunction()
_get_feature_list( _features  ENABLE_LINUX_ETH  ENABLE_GEN_ETH  ENABLE_IPV6  DYNAMIPS_ARCH )
foreach( _feature IN LISTS _features )
   add_definitions( "-D${_feature}" )
endforeach()
string( REPLACE ";" "," _features_commas "${_features}" )
corrosion_import_crate(
   MANIFEST_PATH "${CMAKE_SOURCE_DIR}/rust/Cargo.toml"
   PROFILE release-with-debug
   CRATES
      dynamips-c
      dynamips-c-unstable
   FEATURES ${_features}
)
corrosion_experimental_cbindgen( MANIFEST_PATH "${CMAKE_SOURCE_DIR}/rust/dynamips-c/Cargo.toml" TARGET dynamips_c HEADER_NAME "rust_dynamips_c.h" )
target_include_directories( dynamips_c_unstable INTERFACE "${CMAKE_CURRENT_BINARY_DIR}/corrosion_generated/cbindgen/dynamips_c/include" ) 
add_dependencies( dynamips_c_unstable "_corrosion_cbindgen_dynamips_c_bindings" )

add_custom_target( cargo_fmt COMMAND cargo fmt -v WORKING_DIRECTORY "${CMAKE_SOURCE_DIR}/rust" USES_TERMINAL )
add_custom_target( cargo_clippy COMMAND cargo clippy --profile=release-with-debug --features=${_features_commas} --target-dir "${CMAKE_BINARY_DIR}/cargo/build" -v WORKING_DIRECTORY "${CMAKE_SOURCE_DIR}/rust" USES_TERMINAL )
add_custom_target( cargo_test COMMAND cargo test --profile=release-with-debug --features=${_features_commas} --target-dir "${CMAKE_BINARY_DIR}/cargo/build" -v -- --test-threads 1 WORKING_DIRECTORY "${CMAKE_SOURCE_DIR}/rust" USES_TERMINAL )
add_custom_target( cargo_doc COMMAND cargo doc --features=${_features_commas} --target-dir "${CMAKE_BINARY_DIR}/cargo/build" -v WORKING_DIRECTORY "${CMAKE_SOURCE_DIR}/rust" USES_TERMINAL )

enable_testing()
add_test( NAME test_cargo_fmt COMMAND cargo fmt --check -v WORKING_DIRECTORY "${CMAKE_SOURCE_DIR}/rust" )
add_test( NAME test_cargo_clippy COMMAND cargo clippy --profile=release-with-debug --features=${_features_commas} --target-dir "${CMAKE_BINARY_DIR}/cargo/build" -v -- -D warnings WORKING_DIRECTORY "${CMAKE_SOURCE_DIR}/rust" )
add_test( NAME test_cargo_test COMMAND cargo test --profile=release-with-debug --features=${_features_commas} --target-dir "${CMAKE_BINARY_DIR}/cargo/build" -v -- --test-threads 1 WORKING_DIRECTORY "${CMAKE_SOURCE_DIR}/rust" )

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
