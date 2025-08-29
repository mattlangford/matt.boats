---
date: 2025-08-28
---

# Backups

Over the past year, been self hosting backups of my photos, music, computers, phone, and projects.
This data (where photos and laptop backups are over the past 5 years) only comprises ~500GB of data, which makes me a little sad.

The motivations for self hosting are:
 * It's kind of cool and fun
 * I save $10 a month (and don't think too much about how the cost of my time)
 * Not having to give google/etc all of my data 

Here is (maybe part 1) of a high level overview of how all of this works.

## 3-2-1 backup rule

The general rule is:

 * Maintain three copies of your data.
 * Use two different types of media for storage.
 * Keep at least one copy off-site.

I've been able to accomplish this with:

 * A "working copy" of my data with daily snapshots going back about a month
 * Offline local copy of the data with monthly snapshots
 * Offsite encrypted drive
 * Offsite encrypted backup in Amazon Glacier

## Setup

### MacBook host
I use an old MacBook Pro laptop I have as a host for everything.
This makes phone and laptop backups easy since all of my devices are made by Apple.

### Storage Drives
I have 3 drives connected to the MacBook host:

 * ~2TB HDD (BTRFS) - for a working copy of my data
 * ~5TB HDD (BTRFS) - for monthly snapshots
 * ~500GB SSD (APFS) - used as scratch space for Time Machine/phone backups

I'm using these since they were just laying around (free!). I had started using BTRFS since it looked slick and supported copy-on-write snapshotting.

### Storage VM
This is the main part of the server and is running as a VM on the host machine.

I was hoping to be able to use a docker container for this, but I ran into issues with raw access to the drives since they had to be mounted by the host OS first - which isn't possible for BTRFS on mac.

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
[Navidrome](https://www.navidrome.org) seemed to be a good option for music streaming. It uses the open source Subsonic protocol for querying album/music data - which comes with a good number of open source front ends for the web/iOS.

This also runs as a set of docker containers in the VM. 

### Time Machine
I use the host MacBook to do Time Machine backups onto an external drive.

I had tried [Netatalk](https://netatalk.io/wiki/index.php/Main_Page) as an open source Time Machine destination inside the VM, but ran into pretty consistent issues.

Instead, I periodically (manually) run rsync to get backups from the Time Machine disk to the main drive. This copy _should_ only transfer what is new, so I don't have huge duplicates stored there.

One day I will ditch Time Machine altogether and just run rsync from the other Macs I have.

### Phone
This is similar to the Time Machine backup, I manually run backup copies of my phone using Finder and then rsync that data over to the main drives.

## Encrypted Offsite Backups

Keeping data offsite requires encryption. I use LUKS without a header to create a drive that just looks like random bytes of data. The keys and headers are stored on a USB drive with a hardware based password.

In addition to a drive that I keep offsite, I'll make a copy in AWS. Glacier Deep Archive seems to be the cheapest way to store data in the cloud (since they mainly only charge on egress, which can take a few days).
I've yet needed to restore from a full Glacier backup, but I've successfully performed a few test restores.

I'll do this process once every few months:

 * Perform laptop/phone backups
 * Collect the external drive
 * Mount the encrypted partition using the external USB stick
 * Copy the latest snapshot onto the drive (this is incremental, using BTRFS)
 * Upload a full copy of the encrypted drive to Glacier

