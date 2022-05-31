# Cardstock

## Contributing

Cardstock is written in Rust, using Rocket to deploy the server, and Askama to build the templates for each page. CSS is handrolled, and does not have any external dependencies.

Cardstock uses `git rebase` as a merge strategy -- try and make git commits atomic, and with descriptive comments, and try and rewrite history rather than creating a new commit with additional changes. PRs are welcome.

## roadmap
1. version 0.2
    1. player names - bricks uses sled for this
        1. teams?
        1. ego? season 13 >
    1. better navigation
        1. next/previous season button
        1. some indication of how much of a jump the next will be? 
        1. forward 1 hr? (if possible?)
        1. jump to postseason? election? not sure of nomenclature on this, or on how to do season->time conversion
    1. look and feel
        1. dark mode
        1. change from using plaintext name/uuid to block, like on site
    1. manual dedupe of early season data
        1. the endpoint returned number of fans idoling each player, which we're not using, so there are duplicate board entries with the same arrangement of players.
    1. host on cardstock.sibr.dev?
1. version 0.3
    1. season page - /season/6
1. version 0.4
    1. player page - /player/f70dd57b-55c4-4a62-a5ea-7cc4bf9d8ac1
        1. generate svg graph, like bricks
    1. show annotations on idol board
        1. red line (top 3 season 6/top 10 season 7/top 10 season 8)
        1. feedback icons (6, 11, 18th season 8)
        1. weather icon (season 10)
        1. noodle (season 13 >)
    1. index page
        1. seasons
        1. players
1. future
    1. automatic injest of new data, when blaseball returns (question mark?)

## Community Resources

To get involved with this project, join the [SIBR Discord](https://discord.gg/FfnScUn) server.

To learn more about SIBR, check out the [SIBR FAQ](https://github.com/Society-for-Internet-Blaseball-Research/sibr-faq).

The game of [Blaseball](https://www.blaseball.com) was created by [The Game Band](https://thegameband.com/).
