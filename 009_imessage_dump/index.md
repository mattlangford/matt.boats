---
date: 2025-10-21
---

# iMessage Adventures
_More like iMessage misadventures_

In a similar vein to my [signal backup adventures](../007_signal_dump/signal-conversation-exports.html), I wanted to make sure I had full independent
copies of my iMessage history for querying, to preserve history, and to free up space on my phone. Ideally these wouldn't
require an apple product to open, and could be queried in SQL/python for quick searches.

## The Plan
When I started looking into this I saw a few options

 * Extract messages from phone backups
 * Use third party tools designed to do iMessage backups
 * Clone macOS iMessage database

Since I already make backups of my laptop, I figured trying to tackle the latter option made the most sense.

Accessing the data is fairly easy using `sqlite3`, for example this query shows the timestamp of the latest message
```bash
sqlite3 ~/Library/Messages/chat.db
sqlite> SELECT date FROM message ORDER BY date ASC LIMIT 1;
670960981000000000
```

This timestamp is in nanoseconds since the start of 2001 - this one is roughly `2022/04/06`.

## The Problem

The issue is the oldest message I have on my phone is from _2014_. This means I only have messages on my laptop going
back to when I first powered it up!

Now I had to find a way to sync message from my phone back onto my laptop. Surely this can be done by plugging the phone in?
Unfortunately it doesn't look like it. The only option forward is an iCloud backup middleman.

## The Solution

I purchased enough iCloud space to store a copy of all my messages (remind me to unsubscribe). It took a good amount of
wrestling to get my phone to copy its data up into iCloud. I don't fully understand why, but it would only sync up to about
10,000 message at a time before failing for nondescript reasons.

Since my phone had around 400,000 message on it, I searched to see if there were any troubleshooting tips. Apple forums have
a way of never being very helpful. I tried everything I could find (short of wiping my phone), but resorted to periodically
pressing "Sync Now" in the iMessage iCloud settings. I had to wait for a ~30 minute cool down period after every attempt,
or else the next try would fail immediately. After a couple days of this, all my messages and attachments were in the cloud!


Next up, I had to copy it down to my laptop. This went a lot more smoothly - only two "Sync Now" clicks!  Now for the final
test, I opened up the database and got
```bash
sqlite3 ~/Library/Messages/chat.db
sqlite> SELECT date FROM message ORDER BY date ASC LIMIT 1;
432789997000000000
```

dump roll... `2014/09/19`! The same date as what shows up on my phone. Additionally the whole Messages folder is
about 23GB which lines up nicely with what my phone is using and what iCloud is using.

Success!

