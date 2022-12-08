# Netbox to Windows DHCP Sync

Syncs IPv4 Subnet, Ranges and Reservations from [Netbox](https://github.com/netbox-community/netbox) into a Windows DHCP server.

## Sync

Als Prefixes match the filter will be created as Scope on the DHCP server. Each Prefix needs a corresponding IP-Range which defines the pool.
IP-Addresses matching the filter within a Prefix will be set as reservations.

## Netbox Customisation

### Per Prefix/Subnet lease duration
To set the Prefix/Subnet DHCP lease duration a Integer Custom Field `dhcp_lease_duration` on the Ipam>Prefix can be added to override the default lease duration on a per Prefix/Subnet basis.


### Per Prefix/Subnet DNS Update configuration
To set the Prefix/Subnet DNS settings a Multiple selection Custom Field `dhcp_dns_flags` with the choises `['enabled', 'update_downlevel', 'cleanup_expired', 'update_both_always', 'update_dhcid', 'disable_ptr_update', 'disabled']` on the Ipam>Prefix can be added to override the default on a per Prefix/Subnet basis.

### Reservations without Device
To make a reservation without a Device to assigne the IP-Address to a Text Custom Field `dhcp_reservation_mac` can be added to provide the MAC address.

## Webhook Server

The Hook Server can run as a Windows Servive to listen for WebHooks from Netbox. The Sync is started by an Intervall and on receiving Hooks. Multiple hooks in short succession will only trigger one sync.

## Config

The configfile is read from `C:\ProgramData\netbox_windhcp\netbox_windhcp.cfg` 

```
---
webhook:
    listen: 0.0.0.0:6969
sync:
    dhcp:
        server: dhcp.example.com
        default_dns_flags:
            enabled: true
            cleanup_expired: true
            update_dhcid: true
    netbox:
        apiurl: https://netbox.example.ch/api/
        token: XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
        prefix_filter:
            tag: dhcp
            state: active
        range_filter:
            role: dhcp-pool
            state: active
        reservation_filter:
            tag: dhcp
log:
    dir: C:\ProgramData\netbox_windhcp\
    level: Info
```