---
date: 2025-09-16
---

# Signal Conversation Exports

My friend group uses [Signal](https://signal.org) for most of our messaging needs. I enjoy casual data science - especially when it relates to data in my everyday life - so I was excited when I figured out how to export my conversation data.
 

This write up will be focused on Mac with Signal Desktop.
There are a few resources for how to do this online, but I had trouble getting them to work. This explains the tools I ended up using.

## Database
On Mac, Signal Desktop stores its program files (including encrypted database) in this folder
```
~/Library/Application\ Support/Signal/
```

I've read that the key in 
`Signal/config.json` can be used to unlock the  
`Signal/sql/db.sqlite`, but haven't had luck with this.


## Sigtop
After trying a few tools, [sigtop](https://github.com/tbvdm/sigtop) was the one that I could get to work.

Installing is easy with brew and the [man page is online](https://www.kariliq.nl/man/sigtop.1.html)
```bash
brew install --HEAD tbvdm/tap/sigtop
```

Exporting messages is also pretty easy! This will prompt for keychain access (presumably in order to decrypt the database) and write json versions of messages into the given folder.
```bash
sigtop export-messages -f json /tmp/messages
```

Now my `/tmp/messages` folder is full of conversation data, including DMs and group chats
```bash
$ ls /tmp/messages/ | wc -l
     265
```

## Python
For the most flexibility, consider exporting the full (unencrypted) database
```bash
sigtop export-database /tmp/db.sqlite
```

Then use pandas to open the database, here I'm looking at `messages`
```python
import pandas as pd, sqlite3
with sqlite3.connect("/tmp/db.sqlite") as conn:
    df = pd.read_sql("SELECT * FROM messages", conn)
```

There is quite a lot of data in these tables and not a lot that I've found in the way of documentation... so have fun exploring!

## Ideas
A few ideas and experiments I've done with these backups

 * Creating wordclouds out of DMs (with the help of [wordcloud](https://pypi.org/project/wordcloud/))
 * Plotting who posts the most in a group chat
 * Plotting who gets the most reactions
 * Analyzing the connectivity of your social network - of course this is biased from your view
 * Training an AI to message like you (or one of your friends)
 * Storing memes your friends send
 * Better searching through conversations (with arbitrary filters)
 * Backups


## Disclaimer
Copying signal data like this and storing it in plain text makes it easy to steal all of your conversation data. Try not to leave it in plain text for very long!
