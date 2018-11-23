# Bullet Terminal
For all your terminal-based task-tracking needs

## Save data format
All data is saved to `$XDG_CONFIG_DIR` (or `~/.config/bullet-terminal` when unset). Subdirectories are as follows:

- Local timezone Year (4 digits)
  - Local timezone Month (2 digits, zero-padded)
    - Local timezone Day.txt (2 digits, zero-padded)

It's not stored as utc + offset, so will not handle travel well.

Within each file are your items, one per line (separated by newlines)

## Dear god why?
Its hard to get distracted when buried in a full-screen terminal window.

Honestly? I enjoyed it. It was a silly way to learn rust. While I'm using it to track its own progression, its both missing in the physicality of a real notebook and some alterations I find useful (like projects. And---you know--mobile sync).

Although since they're just easy text files, I could set up an external program to sync them and write a mobile client...

## TODO
[] Break out rendering into proper views: DailyView, MonthlyView, CollectionView, FutureView, etc.
[] Build a smarter data storage solution
[] Give each entry a UUID so they can be linked between dates (say, when scheduled)?
[] Add a calendar view for picking the day (from the title bar)
[] Add a monthly log (should be easy if I change the storage to `$XDG_CONFIG_DIR/bullet-terminal/year/month/day.txt`)
[] Add fst and use it for global search (its overkill but I want to learn the library anyways)
[] Add collection support. It wouldn't be hard to add support for a Monthly collection, and a future log.
[] Add support for custom collections (like tags). Using a proper database would make this easy, but I'm enjoying writing it all myself.
[] Add timezone support. Otherwise it'll become FUBAR if you travel while using it
[] Handle nesting?
