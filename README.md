# Bullet Terminal
For all your terminal-based task-tracking needs

## Save data format
All data is saved to `$XDG_CONFIG_DIR` (or `~/.config/bullet-terminal` when unset). Subdirectories are as follows:

- Local timezone Year (4 digits)
  - Local timezone Month (2 digits, zero-padded)
    - Local timezone Day.txt (2 digits, zero-padded)

It's not stored as utc + offset, so will not handle travel well.

## Dear god why?
Its hard to get distracted when buried in a full-screen terminal window.

Honestly? I enjoyed it. It was a silly way to learn rust. While I'm using it to track its own progression, its both missing in the physicality of a real notebook and some alterations I find useful (like projects. And---you know--mobile sync).

Although since they're just easy text files, I could set up an external program to sync them and write a mobile client...

## TODO
[] Give each entry a UUID so they can be linked between dates (say, when scheduled)
[] Add fst and use it for global search (its overkill but I want to learn the library anyways)
[] Add a calendar view
[] Add a monthly log (should be easy if I change the storage to `$XDG_CONFIG_DIR/bullet-terminal/year/month/day.txt`)
[] Add collection support as a layer below the current "main" layer: the daily view is simply a special collection. It wouldn't be hard to add support for a Monthly collection, and a future log.
[] Add support for custom collections (like tags). Using a proper database would make this easy, but I'm enjoying writing it all myself.
[] Add timezone support. Otherwise it'll become FUBAR if you travel while using it
