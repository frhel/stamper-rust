# Stamper - Rust edition.

A simple CLI tool to assist in management of twitch music streams that have been recorded with OBS,
that will do the following

A Rust rewrite of my node.js tool that I currently use and can be found here:
https://github.com/frhel/stamper

- Create timestamps that are associated with songs from the streamersonglist.com song queue
- Global hotkey to store timestamps to database for each stream / session
- Export session timestamps in a format that is ready to copy-paste into YouTube to create chapters
- Read the current song in queue and open the associated ultimate-guitar link for that song automatically
- Map a global hotkey to mark current song in streamersonglist.com queue as played
- Simple menu to manage songs / timestamps in session

# Progress so far
- Managed to grab the data for the current queue from streamersonglist.com API
- Shaped the data to use in database
- Connect to the streamersonglist.com websocket server and keep the connection alive
- Listen for websocket events to update current song and mark songs as played
