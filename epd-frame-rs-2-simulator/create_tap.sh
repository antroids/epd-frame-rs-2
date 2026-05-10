#!/bin/sh

DEFAULT_NETWORK_INTERFACE=$(ip r | grep "default" | awk '{print $5}' | head -n1)
TAP_NAME="tap99"

sysctl -w net.ipv4.ip_forward=1

iptables -t nat -A POSTROUTING -o "$DEFAULT_NETWORK_INTERFACE" -j MASQUERADE
iptables -A FORWARD -i "$TAP_NAME" -j ACCEPT
iptables -A FORWARD -m conntrack --ctstate RELATED,ESTABLISHED -j ACCEPT

ip tuntap add name "$TAP_NAME" mode tap user $SUDO_USER
ip link set "$TAP_NAME" up
ip addr add 192.168.69.100/24 dev "$TAP_NAME"
ip -6 addr add fe80::100/64 dev "$TAP_NAME"
ip -6 addr add fdaa::100/64 dev "$TAP_NAME"
ip -6 route add fe80::/64 dev "$TAP_NAME"
ip -6 route add fdaa::/64 dev "$TAP_NAME"