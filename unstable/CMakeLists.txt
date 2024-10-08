# dynamips - unstable code

list ( APPEND DYNAMIPS_LIBRARIES dynamips_c_unstable )

set ( COMMON "${CMAKE_SOURCE_DIR}/common" )
set ( LOCAL "${CMAKE_CURRENT_SOURCE_DIR}" )
include_directories ( "${CMAKE_CURRENT_SOURCE_DIR}" "${COMMON}" "${CMAKE_CURRENT_BINARY_DIR}" )

#-------------------------#
# dynamips: unstable code #
#-------------------------#

if ( NOT BUILD_DYNAMIPS_UNSTABLE )
   return ()
endif ( NOT BUILD_DYNAMIPS_UNSTABLE )

# dynamips_*_unstable
add_definitions( "-DUSE_UNSTABLE" )

set ( _files
   "${COMMON}/utils.c"
   "${COMMON}/frame_relay.c"
   "${COMMON}/eth_switch.c"
   "${COMMON}/dynamips.c"
   "${LOCAL}/vm.c"
   "${LOCAL}/cpu.c"
   "${LOCAL}/tcb.c" # only present in unstable
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

# dynamips_amd64_unstable
if ( "amd64" STREQUAL "${DYNAMIPS_ARCH}" )
add_executable ( dynamips_amd64_unstable
   ${_files}
   "${LOCAL}/mips64_amd64_trans.c"
   "${LOCAL}/ppc32_amd64_trans.c"
   )
target_link_libraries ( dynamips_amd64_unstable ${DYNAMIPS_LIBRARIES} )
maybe_rename_to_dynamips ( dynamips_amd64_unstable )
install_executable ( dynamips_amd64_unstable )
endif ()

# dynamips_x86_unstable
if ( "x86" STREQUAL "${DYNAMIPS_ARCH}" )
add_executable ( dynamips_x86_unstable
   ${_files}
   "${LOCAL}/mips64_x86_trans.c"
   "${LOCAL}/ppc32_x86_trans.c"
   )
target_link_libraries ( dynamips_x86_unstable ${DYNAMIPS_LIBRARIES} )
maybe_rename_to_dynamips ( dynamips_x86_unstable )
install_executable ( dynamips_x86_unstable )
endif ()

# dynamips_nojit_unstable
if ( "nojit" STREQUAL "${DYNAMIPS_ARCH}" )
add_executable ( dynamips_nojit_unstable
   ${_files}
   "${LOCAL}/mips64_nojit_trans.c"
   "${COMMON}/ppc32_nojit_trans.c"
   )
target_link_libraries ( dynamips_nojit_unstable ${DYNAMIPS_LIBRARIES} )
maybe_rename_to_dynamips ( dynamips_nojit_unstable )
install_executable ( dynamips_nojit_unstable )
endif ()
