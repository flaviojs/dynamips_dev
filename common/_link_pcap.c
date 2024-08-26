
#include <pcap.h>

/*
TODO fix linking

After moving pcap_* calls from C to rust, it stopped linking to pcap.
For now keep a call around to make sure it links.
*/
void link_pcap(void) {
    pcap_strerror(0);
}
