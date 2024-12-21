# dynamips - utility functions

# standard checks
include ( CheckCCompilerFlag )
include ( CheckCSourceCompiles )
include ( CheckCSourceRuns )
include ( CheckCXXCompilerFlag )
include ( CheckCXXSourceCompiles )
include ( CheckCXXSourceRuns )
include ( CheckFunctionExists )
include ( CheckIncludeFile )
include ( CheckIncludeFileCXX )
include ( CheckIncludeFiles )
include ( CheckLibraryExists )
include ( CheckPrototypeDefinition )
include ( CheckStructHasMember )
include ( CheckSymbolExists )
include ( CheckTypeSize )
include ( CheckVariableExists )

# print variables
function ( print_variables )
   foreach ( _var ${ARGV} )
      message ( STATUS "${_var}=${${_var}}" )
   endforeach ()
endfunction ( print_variables )

# convert the name to a standard variable name
macro ( standard_variable_name _var _name )
      string ( REGEX REPLACE "[^a-zA-Z0-9]" "_" ${_var} "${_name}" )
      string ( TOUPPER "${${_var}}" ${_var} )
endmacro ()

# check if we can compile with the library for the target architecture
macro ( check_arch_library _var _func _header _libvar )
   set ( _n 0 )
   set ( ${_var} )
   foreach ( _lib "${${_libvar}}" ${ARGN} )
      check_library_exists ( "${_lib}" ${_func} ${_header} ${_var}_${_n} )
      if ( ${_var}_${_n} )
         # success
         set ( ${_var} 1 )
         if ( NOT "${_lib}" STREQUAL "${${_libvar}}" )
            set ( ${_libvar} ${_lib} )
            print_variables ( ${_libvar} )
         endif ()
         break ()
      endif ()
      math ( EXPR _n "${_n}+1" )
   endforeach ()
endmacro ()

# could not compile with the library for the target architecture
macro ( bad_arch_library _type _lib _vars )
   message (
      ${_type} 
      "${_lib} was found but cannot be used with DYNAMIPS_ARCH=${DYNAMIPS_ARCH}. "
      "Make sure the library for the target architecture is installed. "
      "If needed, you can set the variables ${_vars} manually. "
      )
endmacro ()

# rename target DYNAMIPS_RENAME_TARGET to dynamips
macro ( maybe_rename_to_dynamips _target )
   if ( "${_target}" STREQUAL "${DYNAMIPS_RENAME_TARGET}" )
      set_target_properties ( ${_target} PROPERTIES OUTPUT_NAME "dynamips" )
   endif()
endmacro ( maybe_rename_to_dynamips _target )

# install executables
function ( install_executable _target )
   install (
      TARGETS ${_target}
      RUNTIME DESTINATION "${CMAKE_INSTALL_BINDIR}"
      COMPONENT "executables"
      )
endfunction ( install_executable )

# install docs
function ( install_docs )
   if ( ${ARGC} LESS 1 )
     # no docs
     return ()
   endif ()
   install (
      FILES ${ARGV}
      DESTINATION "${CMAKE_INSTALL_DOCDIR}"
      COMPONENT "docs"
      )
endfunction ( install_docs )

# install man pages
function ( install_man_pages )
   if ( ${ARGC} LESS 1 )
     # no man pages
     return ()
   endif ()
   foreach ( _file ${ARGV} )
      string ( REGEX REPLACE "^.*\\." "" _page "${_file}" )
      if ( NOT "${_page}" MATCHES "[0-9]+" )
         message ( FATAL_ERROR "not a man page: ${_file}" )
      endif ()
      install (
         FILES "${_file}"
         DESTINATION "${CMAKE_INSTALL_MANDIR}/man${_page}"
         COMPONENT "docs"
         )
   endforeach ()
endfunction ( install_man_pages )

# Creates a cargo crate and uses cargo check to probe if something compiles.
#  - varname
#  - CFG "has_xxx"
#  - DEPENDENCIES "" (optional)
#  - CODE "" (optional)
#  - OPTION
function ( cargo_probe varname )
   set ( options )
   set ( oneValueArgs CFG DEPENDENCIES CODE )
   set ( multiValueArgs )
   cmake_parse_arguments ( PARSE_ARGV 0 PROBE "${options}" "${oneValueArgs}" "${multiValueArgs}" )
   if ( NOT PROBE_CFG )
      message ( FATAL_ERROR "CFG is required" )
   endif()
   message ( STATUS "Probing ${PROBE_CFG}" )
   message ( VERBOSE "DEPENDENCIES: ${PROBE_DEPENDENCIES}" )
   message ( VERBOSE "CODE: ${PROBE_CODE}" )
   set ( _in_dir "${CMAKE_CURRENT_LIST_DIR}/cargo_probe.in" )
   set ( _crate_dir "${CMAKE_BINARY_DIR}/cargo_probe/${PROBE_CFG}" )
   file( GLOB_RECURSE _paths RELATIVE "${_in_dir}" CONFIGURE_DEPENDS "${_in_dir}/*" )
   foreach ( _path IN ITEMS ${_paths} )
      message( VERBOSE "Configure ${_in_dir}/${_path} -> ${_crate_dir}/${_path}" )
      configure_file ( "${_in_dir}/${_path}" "${_crate_dir}/${_path}" @ONLY )
   endforeach()
   execute_process (
      COMMAND cargo check --target-dir "${CMAKE_BINARY_DIR}/cargo/probe" -v
      WORKING_DIRECTORY "${_crate_dir}"
      RESULT_VARIABLE _result
      OUTPUT_VARIABLE _stdout
      ERROR_VARIABLE _stderr
   )
   message ( VERBOSE "result: =${_result}" )
   message ( VERBOSE "stdout:\n${_stdout}" )
   message ( VERBOSE "stderr:\n${_stderr}" )
   message ( FATAL_ERROR "TODO" )
endfunction ( )
