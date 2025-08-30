---
date: 2025-08-28
---

# Backups
_If it's worth saving, it's worth over saving_

Over the past year, I started self hosting backups of my photos, music, computers, phones, and projects.
This data, spanning over five years of my life, comprises a meager ~500GB. 

My motivations for self hosting are:

 * It's kind of cool and fun
 * I save $10 a month (best not to think too much about the cost of my time)
 * Not giving google/etc all of my data 

Here is (maybe part 1) of a high level overview of how all of this works.

## 3-2-1 backup rule

The general rule is:

 * Maintain *three* copies of your data
 * Use *two* different types of media for storage
 * Keep at least *one* copy off-site

I've accomplished this with:

 * A "working copy" of my data with daily snapshots going back about a month
 * Offline local copy of the data with monthly snapshots
 * Offsite encrypted drive
 * Offsite encrypted backup in Amazon Glacier

## Setup

### MacBook host
I use an old MacBook Pro laptop as a host for everything.
This makes phone and laptop backups easy since all of my devices are made by Apple.

### Storage Drives
I have 3 drives connected to the MacBook host:

 * ~2TB HDD (BTRFS) - a working copy of my data
 * ~5TB HDD (BTRFS) - monthly snapshots
 * ~500GB SSD (APFS) - scratch space for Time Machine/phone backups

These were just hard drives I had laying around (free!) and I started using BTRFS since it looked slick and supported copy-on-write snapshotting.

### Storage VM
This is the main part of the server and is running as a VM on the host machine.

I was hoping to be able to use a docker container for this, but I ran into issues with raw access to the drives since they had to be mounted by the host OS first - which isn't possible for BTRFS on Mac.

## Services

I use tailscale as a VPN so I always have access to the data.

There are a couple of git servers hosted on here too for various local projects.

For status updates, I generate a page every few minutes which contains usage information, drive health, and CPU/memory usage.

### Immich
[Immich](http://immich.app) is an open source image hosting service. This was super easy to set up and has been working very well. It provides features on par with google/apple photos (face/object detection, live photos, map information).

This runs with docker compose inside of the Storage VM and I'm able to connect to the open port inside my tailnet.

Make sure to keep snapshots or other backups since they say

> Do not use it as the only way to store your photos and videos

### Navidrome
[Navidrome](https://www.navidrome.org) is a good open source option for music streaming. It uses the Subsonic protocol for querying album/music data which comes with a good number of front ends for the web/iOS.

This also runs as a set of docker containers in the VM. 

### Time Machine
I use the host MacBook to do Time Machine backups onto an external drive.

I tried [Netatalk](https://netatalk.io/wiki/index.php/Main_Page) as an open source Time Machine destination inside the VM, but ran into pretty consistent issues.

Instead, I periodically (manually) run rsync to get backups from the Time Machine disk to the main drive. This copy _should_ only transfer what is new, so I don't have huge duplicates stored there.

One day I will ditch Time Machine altogether and just run rsync from the other Macs I have.

### Phone
This is similar to the Time Machine backup, I manually run backup copies of my phone using Finder and then rsync that data over to the main drives.

## Encrypted Offsite Backups

Keeping data offsite requires encryption. I use LUKS without a header to create a drive that just looks like random bytes of data. The keys and headers are stored on a USB drive with a hardware based password.

In addition to a drive that I keep offsite, I make a copy in AWS. Glacier Deep Archive is the cheapest way to store data in the cloud (since they mainly only charge on egress, which can take a few days).
I have yet needed to restore from a full Glacier backup, but I've successfully performed a few test restores.

I'll do this process once every few months:

 * Perform laptop/phone backups
 * Collect the external drive
 * Mount the encrypted partition using the external USB stick
 * Copy the latest snapshot onto the drive (this is incremental, using BTRFS)
 * Upload a full copy of the encrypted drive to Glacier

