# dynamips - stable code + tools

list ( APPEND DYNAMIPS_LIBRARIES dynamips_c )

set ( COMMON "${CMAKE_SOURCE_DIR}/common" )
set ( LOCAL "${CMAKE_CURRENT_SOURCE_DIR}" )
include_directories ( "${CMAKE_CURRENT_SOURCE_DIR}" "${COMMON}" "${CMAKE_CURRENT_BINARY_DIR}" )

# udp_send
if ( BUILD_UDP_SEND )
install ( IMPORTED_RUNTIME_ARTIFACTS udp_send ) # TODO IMPORTED_RUNTIME_ARTIFACTS requires cmake 3.21
endif ( BUILD_UDP_SEND )

# udp_recv
if ( BUILD_UDP_RECV )
install ( IMPORTED_RUNTIME_ARTIFACTS udp_recv ) # TODO IMPORTED_RUNTIME_ARTIFACTS requires cmake 3.21
endif ( BUILD_UDP_RECV )

# nvram_export
if ( BUILD_NVRAM_EXPORT )
install ( IMPORTED_RUNTIME_ARTIFACTS nvram_export ) # TODO IMPORTED_RUNTIME_ARTIFACTS requires cmake 3.21
endif ( BUILD_NVRAM_EXPORT )

#-----------------------#
# dynamips: stable code #
#-----------------------#

if ( NOT BUILD_DYNAMIPS_STABLE )
   return ()
endif ( NOT BUILD_DYNAMIPS_STABLE )

# set MAC64HACK on stable OSX amd64 build
if ( APPLE AND "amd64" STREQUAL "${DYNAMIPS_ARCH}" )
   add_definitions( "-DMAC64HACK" )
endif()


# dynamips_*_stable
set ( _files
   "${COMMON}/utils.c"
   "${COMMON}/frame_relay.c"
   "${COMMON}/eth_switch.c"
   "${COMMON}/dynamips.c"
   "${LOCAL}/vm.c"
   "${LOCAL}/cpu.c"
   "${LOCAL}/mips64.c"
   "${LOCAL}/mips64_mem.c"
   "${LOCAL}/mips64_jit.c"
   "${LOCAL}/mips64_exec.c"
   "${LOCAL}/ppc32.c"
   "${LOCAL}/ppc32_mem.c"
   "${LOCAL}/ppc32_jit.c"
   "${LOCAL}/ppc32_exec.c"
   "${LOCAL}/ppc32_vmtest.c"
   "${COMMON}/dev_mpc860.c"
   "${COMMON}/dev_mv64460.c"
   "${COMMON}/dev_plx.c"
   "${COMMON}/dev_dec21140.c"
   "${COMMON}/dev_i8254x.c"
   "${COMMON}/dev_i8255x.c"
   "${COMMON}/dev_mueslix.c"
   "${COMMON}/dev_wic_serial.c"
   "${COMMON}/dev_c3600.c"
   "${COMMON}/dev_c3600_bay.c"
   "${COMMON}/dev_c3600_iofpga.c"
   "${COMMON}/dev_c3600_eth.c"
   "${COMMON}/dev_c3600_serial.c"
   "${COMMON}/dev_c7200.c"
   "${COMMON}/dev_c7200_iofpga.c"
   "${COMMON}/dev_c7200_mpfpga.c"
   "${COMMON}/dev_c7200_eth.c"
   "${COMMON}/dev_c7200_serial.c"
   "${COMMON}/dev_c7200_pos.c"
   "${COMMON}/dev_c7200_bri.c"
   "${COMMON}/dev_c7200_jcpa.c"
   "${COMMON}/dev_c2691.c"
   "${COMMON}/dev_c2691_iofpga.c"
   "${COMMON}/dev_c2691_eth.c"
   "${COMMON}/dev_c2691_serial.c"
   "${COMMON}/dev_c2691_wic.c"
   "${COMMON}/dev_c2691_pcmod.c"
   "${COMMON}/dev_c3725.c"
   "${COMMON}/dev_c3725_iofpga.c"
   "${COMMON}/dev_c3725_eth.c"
   "${COMMON}/dev_c3725_serial.c"
   "${COMMON}/dev_c3725_wic.c"
   "${COMMON}/dev_c3725_pcmod.c"
   "${COMMON}/dev_c3745.c"
   "${COMMON}/dev_c3745_iofpga.c"
   "${COMMON}/dev_c3745_eth.c"
   "${COMMON}/dev_c3745_serial.c"
   "${COMMON}/dev_c3745_wic.c"
   "${COMMON}/dev_c3745_pcmod.c"
   "${COMMON}/dev_c2600.c"
   "${COMMON}/dev_c2600_pci.c"
   "${COMMON}/dev_c2600_iofpga.c"
   "${COMMON}/dev_c2600_eth.c"
   "${COMMON}/dev_c2600_pcmod.c"
   "${COMMON}/dev_c2600_wic.c"
   "${COMMON}/dev_c1700.c"
   "${COMMON}/dev_c1700_iofpga.c"
   "${COMMON}/dev_c1700_eth.c"
   "${COMMON}/dev_c1700_wic.c"
   "${COMMON}/dev_c6msfc1.c"
   "${COMMON}/dev_c6msfc1_iofpga.c"
   "${COMMON}/dev_c6msfc1_mpfpga.c"
   "${COMMON}/dev_c6sup1.c"
   "${COMMON}/dev_c6sup1_iofpga.c"
   "${COMMON}/dev_c6sup1_mpfpga.c"
   "${COMMON}/dev_nm_16esw.c"
   "${COMMON}/dev_pa_a1.c"
   "${COMMON}/dev_pa_mc8te1.c"
   "${LOCAL}/hypervisor.c"
   "${COMMON}/hv_nio.c"
   "${COMMON}/hv_nio_bridge.c"
   "${COMMON}/hv_frsw.c"
   "${COMMON}/hv_atmsw.c"
   "${COMMON}/hv_atm_bridge.c"
   "${COMMON}/hv_ethsw.c"
   "${LOCAL}/hv_vm.c"
   "${COMMON}/hv_vm_debug.c"
   "${COMMON}/hv_store.c"
   "${COMMON}/hv_c7200.c"
   "${COMMON}/hv_c3600.c"
   "${COMMON}/hv_c2691.c"
   "${COMMON}/hv_c3725.c"
   "${COMMON}/hv_c3745.c"
   "${COMMON}/hv_c2600.c"
   "${COMMON}/hv_c1700.c"
   "${COMMON}/get_cpu_time.c"
   "${COMMON}/dev_lxt970a.c"
   )
if ( ENABLE_GEN_ETH )
   set ( _files ${_files} "${COMMON}/_link_pcap.c" )
endif ( ENABLE_GEN_ETH )

# dynamips_amd64_stable
if ( "amd64" STREQUAL "${DYNAMIPS_ARCH}" )
add_executable ( dynamips_amd64_stable
   ${_files}
   "${LOCAL}/mips64_amd64_trans.c"
   "${LOCAL}/ppc32_amd64_trans.c"
   )
target_link_libraries ( dynamips_amd64_stable ${DYNAMIPS_LIBRARIES} )
maybe_rename_to_dynamips ( dynamips_amd64_stable )
install_executable ( dynamips_amd64_stable )
endif ()

# dynamips_x86_stable
if ( "x86" STREQUAL "${DYNAMIPS_ARCH}" )
add_executable ( dynamips_x86_stable
   ${_files}
   "${LOCAL}/mips64_x86_trans.c"
   "${LOCAL}/ppc32_x86_trans.c"
   )
target_link_libraries ( dynamips_x86_stable ${DYNAMIPS_LIBRARIES} )
maybe_rename_to_dynamips ( dynamips_x86_stable )
install_executable ( dynamips_x86_stable )
endif ()

# dynamips_nojit_stable
if ( "nojit" STREQUAL "${DYNAMIPS_ARCH}" )
add_executable ( dynamips_nojit_stable
   ${_files}
   "${LOCAL}/mips64_nojit_trans.c"
   "${COMMON}/ppc32_nojit_trans.c"
   )
target_link_libraries ( dynamips_nojit_stable ${DYNAMIPS_LIBRARIES} )
maybe_rename_to_dynamips ( dynamips_nojit_stable )
install_executable ( dynamips_nojit_stable )
endif ()
