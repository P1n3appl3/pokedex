## Why does this exist

I'm Feeling Restless at work. I haven't gotten to write a non-trivial amount of rust recently. I've also been doing a LOT of small tweaks to various software I use as I bring up my new laptop and I'm exhausted from that and want to stop worrying about those big programs and their various shortcomings by writing a teeny program that perfectly satisfies my use case.

[eevee's blog post](https://eev.ee/blog/2017/08/02/datamining-pokemon/) does this for the full-size sprites, but I want the smaller menu/box sprites to display in my terminal. once I realized this I decided to go do it myself for fun. [eevee's toolkit](https://github.com/eevee/porigon-z/blob/master/porigonz/nds/util/sprites.py) for data extraction (or the [hg/ss decompilation](https://github.com/pret/pokeheartgold/blob/master/include/constants/sprites.h)) can probably get those icon-sized sprites too, but I didn't bother checking.

[This](https://github.com/Tenchi2xh/pokedex-cli) one had the coolest UI out of the dozen or so I looked at, so it's what I'll try to recreate. That means I need not only sprites, but names, nicknames, evolutions, types, height/weight, and pokedex prose.

``` shell
🍍 file diamond.nds && md5sum diamond.nds
diamond.nds: Nintendo DS ROM image: "POKEMON D" (ADAE01, Rev.05) (decrypted)
02a1af2a677d101394b1d99164a8c249  diamond.nds
```

Side note: it's cool that `file` has support for these ROMs, I went and [found the source](https://github.com/file/file/blame/7f9a0931f46282ab158c4a8a9474fba6611baba0/magic/Magdir/console#L475) because I was pleasantly surprised that it just worked.

## Scope Creep

There's lots of pokemon games and they've got lots of art/data, but the _maximum_ I'd be interested in covering are:

Games:
- Fire Red/Leaf Green
- Ruby/Sapphire/Emerald
- Heart Gold/Soul Silver
- Diamond/Pearl/Platinum
- maybe b/w/b2/w2?? after i play them?

Info (priority list):
- box sprites
- front sprites
- pokedex prose description
- pokedex attributes (height/weight)
- type/stats
- evolutions
- hg/ss overworld sprites (2 frames for all 4 directions)
- back sprites
- pokeball/other item sprites
- abilities?
- moves? learnset/teachset/description/power/acc/probabilities

Trainer teams with full level/stat/moveset info and the trainersprite would be cool but wayyyy too much work

remember to double check for exact sprite re-use (I've heard some platinum sprites just swapped frame 1/2)

Functionality:
- dex entries:
  - name/nick/num
  - height/weight
  - sprite, change the pool based on dimensions
    - random version or arg to lock a gen/game/type/shiny
  - type/strengths/weaknesses (including 0/4x)
  - stats
  - evolution chart
  - 2 frame box sprite animations seem doable, and I'll at least get static versions of both large frames but probably not the more dynamic animations from emerald and onward.
  - ability/learnset
- type chart
- region
- cli
  - hyphen option completion, otherwise name/number(id) and gender/form for second positional arg

### Ideas

- levenshein distance or fuzzy search for names?
- [natures grid](https://pokemondb.net/mechanics/natures)
- [type chart](https://pokemondb.net/type/old)
  - single and double type effectivenesses
- move coverage calculator?
- search for mons that can learn certain move(s) alongside certain abilities with certain type(s), filter thresholds or sort by stats?
  - [this cli pokedex](https://github.com/purplecraft999/rsdex) has some cute filter options

## References

- great historical background and technical breakdown of the [gba](https://www.copetti.org/writings/consoles/game-boy-advance/) and [nds](https://www.copetti.org/writings/consoles/nintendo-ds)
- comprehensive gba/ds technical resource including file formats: [GBATEK](https://problemkaputt.de/gbatek.htm)
- [gen 3 color palettes](https://voliol.neocities.org/articles/genIIIpalettes) explanation
- hgss had an unnamed filesystem but some resources like [this](https://projectpokemon.org/home/docs/gen-4/hgss-file-system-r21/) have guides to which file is which
- data format [notes and dumps](https://projectpokemon.org/rawdb) for all gen 4 games
- another [ds rom header breakdown](http://dsibrew.org/wiki/DSi_cartridge_header) with the extra dsi bits (maybe relevant if I ever want to support b/w/b2/w2)
- [list](https://gist.github.com/eevee/15a92e26088d79a77fca) of pokemon with forms
- [uxie](https://github.com/KalaayPT/uxie) is a project that implements most of the functionality I'd like to write. If I weren't writing this tool for fun I'd just use this as a library.

eevee's blog posts:
- https://eev.ee/blog/2016/08/05/storing-pok%C3%A9mon-without-sql
- https://eev.ee/blog/2017/08/02/datamining-pokemon

tools and resources to read/dump nitro filesystem:
- high quality rust [library + cli](https://lib.rs/crates/ds-rom)
- another [rust library](https://lib.rs/crates/nitro_fs) and a [less complete one](https://lib.rs/crates/nds)
- https://github.com/pret/ds_disassembly_tools
- https://scenegate.github.io/Ekona/
- cute [blog post](https://imrannazar.com/The-Smallest-NDS-File) on making the smallest nds file possible

tools to dump narcs:
- https://github.com/nickworonekin/narchive
- https://ndspy.readthedocs.io/en/latest/api/narc.html

hacking tools/editors that probably already have all the info I want:
- https://github.com/magical/pokemon-nds-sprites for gen4 sprites
- https://github.com/haven1433/HexManiacAdvance for gen3
- https://github.com/turtleisaac/PokEditor for gen4

other terminal pokemon sprite renderers:
- https://github.com/Tenchi2xh/pokedex-cli
- https://github.com/yannjor/krabby and [this fork](https://github.com/altair-39/kingler)
- https://github.com/talwat/pokeget-rs
- https://github.com/31marcosalsa/pokeTerm
- https://github.com/Findarato/pokemon-colorscripts
- https://github.com/possatti/pokemonsay
- https://github.com/HRKings/pokemonsay-newgenerations

msg file string parsing:
- https://github.com/yakoswg/chatot
- https://codeberg.org/AyashiBox/pkmn-strings

possible online sources for fallback data that I don't derive like encounter tables:
- veekun (TODO: check if their database has this and maybe use it offline?)
- serebii
- bulbapedia
- pokemondb
- pokeapi using [either](https://github.com/zitronenjoghurt/poke-data) of [these](https://github.com/mlemesle/rustemon) libraries

existing sprite databases to check against:
- [veekun's](http://git.veekun.com/pokedex-media.git/tree/HEAD:/pokemon/icons) (they also have a convenient [separate download](https://veekun.com/dex/downloads))
- [pokesprite](https://msikma.github.io/pokesprite/)
- pokemondb/pokemondb/serebii have structured urls so i could scrape

## notes

- msg.narc (checked in diamond, soul silver, and platinum)
  - 16 names
  - 190 natures
  - 192 pokemon nickname suggestions
  - 205 statuses
  - 310 names... japanese?
  - 326 trade pokemon nicknames
  - 343 item desc
  - 344 item
  - 362 POKEMON
  - 373 berry types
  - 382 place names
  - 414 poffin types
  - 495 stat names
  - 552 abilities
  - 554 ability descriptions
  - 559 trainers
  - 560 trainer titles
  - 565 types
  - 588 moves
  - 615 pokedex entry
  - 619 weight
  - 620 height
  - 621 "___ pokemon"
