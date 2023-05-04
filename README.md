# ripcalc

ipcalc in Rust

## Modes

### Show network

This is the default mode of operation and shows information about the given network.

    ripcalc 192.168.2.4/24

outputs

    Address:   192.168.2.4          11000000.10101000.00000010.00000100
    Netmask:   255.255.255.0 = 24   11111111.11111111.11111111.00000000
    Wildcard:  0.0.0.255            00000000.00000000.00000000.11111111
    =>
    Network:   192.168.2.0/24       11000000.10101000.00000010.00000000
    HostMin:   192.168.2.1          11000000.10101000.00000010.00000001
    HostMax:   192.168.2.254        11000000.10101000.00000010.11111110
    Broadcast: 192.168.2.255        11000000.10101000.00000010.11111111
    Hosts/Net: 254                  Class C

IPv6 is supported as well;

    ripcalc fe80::38a2:b5f3:58dc:13d9/10

outputs

    Address:   fe80::38a2:b5f3:58dc:13d9                     1111111010000000:0000000000000000:0000000000000000:0000000000000000:0011100010100010:1011010111110011:0101100011011100:0001001111011001
    Netmask:   ffc0:: = 10                                   1111111111000000:0000000000000000:0000000000000000:0000000000000000:0000000000000000:0000000000000000:0000000000000000:0000000000000000
    Wildcard:  3f:ffff:ffff:ffff:ffff:ffff:ffff:ffff         0000000000111111:1111111111111111:1111111111111111:1111111111111111:1111111111111111:1111111111111111:1111111111111111:1111111111111111
    =>
    Network:   fe80::/10                                     1111111010000000:0000000000000000:0000000000000000:0000000000000000:0000000000000000:0000000000000000:0000000000000000:0000000000000000
    HostMin:   fe80::1                                       1111111010000000:0000000000000000:0000000000000000:0000000000000000:0000000000000000:0000000000000000:0000000000000000:0000000000000001
    HostMax:   febf:ffff:ffff:ffff:ffff:ffff:ffff:fffe       1111111010111111:1111111111111111:1111111111111111:1111111111111111:1111111111111111:1111111111111111:1111111111111111:1111111111111110
    Broadcast: febf:ffff:ffff:ffff:ffff:ffff:ffff:ffff       1111111010111111:1111111111111111:1111111111111111:1111111111111111:1111111111111111:1111111111111111:1111111111111111:1111111111111111
    Hosts/Net: 332306998946228968225951765070086142

### Minimize

Takes multiple networks and attempts to merge neighboring ones into larger networks, minimizing the
list of networks without including addresses not contained in the original networks.

    ripcalc --minimize 192.168.2.0/24 192.168.3.0/24 192.168.3.128/25 192.168.4.128/25

outputs

    192.168.2.0/23
    192.168.4.128/25

### Split

Takes a network and multiple integers and splits the network into the smallest possible slices to
house the number of hosts given by the integers.

    ripcalc --split 192.168.2.0/24 5 16 5 5

outputs

    Subnet to split:
    Network:   192.168.2.0/24       11000000.10101000.00000010.00000000
    HostMin:   192.168.2.1          11000000.10101000.00000010.00000001
    HostMax:   192.168.2.254        11000000.10101000.00000010.11111110
    Broadcast: 192.168.2.255        11000000.10101000.00000010.11111111
    Hosts/Net: 254                  Class C

    Subnet for 5 hosts:
    Network:   192.168.2.32/29      11000000.10101000.00000010.00100000
    HostMin:   192.168.2.33         11000000.10101000.00000010.00100001
    HostMax:   192.168.2.38         11000000.10101000.00000010.00100110
    Broadcast: 192.168.2.39         11000000.10101000.00000010.00100111
    Hosts/Net: 6                    Class C

    Subnet for 16 hosts:
    Network:   192.168.2.0/27       11000000.10101000.00000010.00000000
    HostMin:   192.168.2.1          11000000.10101000.00000010.00000001
    HostMax:   192.168.2.30         11000000.10101000.00000010.00011110
    Broadcast: 192.168.2.31         11000000.10101000.00000010.00011111
    Hosts/Net: 30                   Class C

    Subnet for 5 hosts:
    Network:   192.168.2.40/29      11000000.10101000.00000010.00101000
    HostMin:   192.168.2.41         11000000.10101000.00000010.00101001
    HostMax:   192.168.2.46         11000000.10101000.00000010.00101110
    Broadcast: 192.168.2.47         11000000.10101000.00000010.00101111
    Hosts/Net: 6                    Class C

    Subnet for 5 hosts:
    Network:   192.168.2.48/29      11000000.10101000.00000010.00110000
    HostMin:   192.168.2.49         11000000.10101000.00000010.00110001
    HostMax:   192.168.2.54         11000000.10101000.00000010.00110110
    Broadcast: 192.168.2.55         11000000.10101000.00000010.00110111
    Hosts/Net: 6                    Class C

    Unused networks:
    192.168.2.56/29
    192.168.2.64/26
    192.168.2.128/25

### Resize

Resizes the given network to a supernet or a set of subnets that match the given CIDR prefix or subnet mask.

    ripcalc --resize 192.168.2.0/24 255.255.255.192

outputs

    Original network:
    Network:   192.168.2.0/24       11000000.10101000.00000010.00000000
    HostMin:   192.168.2.1          11000000.10101000.00000010.00000001
    HostMax:   192.168.2.254        11000000.10101000.00000010.11111110
    Broadcast: 192.168.2.255        11000000.10101000.00000010.11111111
    Hosts/Net: 254                  Class C

    Subnet 1:
    Network:   192.168.2.0/26       11000000.10101000.00000010.00000000
    HostMin:   192.168.2.1          11000000.10101000.00000010.00000001
    HostMax:   192.168.2.62         11000000.10101000.00000010.00111110
    Broadcast: 192.168.2.63         11000000.10101000.00000010.00111111
    Hosts/Net: 62                   Class C

    Subnet 2:
    Network:   192.168.2.64/26      11000000.10101000.00000010.01000000
    HostMin:   192.168.2.65         11000000.10101000.00000010.01000001
    HostMax:   192.168.2.126        11000000.10101000.00000010.01111110
    Broadcast: 192.168.2.127        11000000.10101000.00000010.01111111
    Hosts/Net: 62                   Class C

    Subnet 3:
    Network:   192.168.2.128/26     11000000.10101000.00000010.10000000
    HostMin:   192.168.2.129        11000000.10101000.00000010.10000001
    HostMax:   192.168.2.190        11000000.10101000.00000010.10111110
    Broadcast: 192.168.2.191        11000000.10101000.00000010.10111111
    Hosts/Net: 62                   Class C

    Subnet 4:
    Network:   192.168.2.192/26     11000000.10101000.00000010.11000000
    HostMin:   192.168.2.193        11000000.10101000.00000010.11000001
    HostMax:   192.168.2.254        11000000.10101000.00000010.11111110
    Broadcast: 192.168.2.255        11000000.10101000.00000010.11111111
    Hosts/Net: 62                   Class C

### Enumerate

Outputs all addresses in the given network (including the network and the broadcast address). Using
this with an IPv6 network might not be particularly enjoyable.

    ripcalc --enumerate 192.168.2.0/28

outputs

    192.168.2.0
    192.168.2.1
    192.168.2.2
    192.168.2.3
    192.168.2.4
    192.168.2.5
    192.168.2.6
    192.168.2.7
    192.168.2.8
    192.168.2.9
    192.168.2.10
    192.168.2.11
    192.168.2.12
    192.168.2.13
    192.168.2.14
    192.168.2.15

## Special features

### "Lopsided" networks

While most networking stacks nowadays expect networks to be defined such that all host bits follow
all network bits (when enumerated from most to least significant), all modes of ripcalc except
`--split` can perform network calculations where this is not the case. For example,

    ripcalc --enumerate 192.168.0.2/255.255.240.255

outputs

    192.168.0.2
    192.168.1.2
    192.168.2.2
    192.168.3.2
    192.168.4.2
    192.168.5.2
    192.168.6.2
    192.168.7.2
    192.168.8.2
    192.168.9.2
    192.168.10.2
    192.168.11.2
    192.168.12.2
    192.168.13.2
    192.168.14.2
    192.168.15.2

### Network syntax

Networks can be specified as follows:

* CIDR syntax: `192.168.2.0/24`
* subnet mask syntax: `192.168.2.0/255.255.255.0`
* Cisco wildcard syntax (complement of subnet mask; specified with a leading minus): `192.168.2.0/-0.0.0.255`

Those three networks are equivalent. Note that lopsided networks, as introduced in the previous
section, cannot be specified using CIDR syntax.
