I'm Feeling restless at work, haven't gotten to write a non-trivial amount of rust recently. I've also been doing a LOT of small tweaks to various software I use as I bring up my new laptop and I'm exhausted from that and want to stop worrying about those big programs and their various shortcomings by writing a teeny program that perfectly satisfies my use case.

[eevee's blog post](https://eev.ee/blog/2017/08/02/datamining-pokemon/) does this for the full-size sprites, but I want the smaller menu/box sprites to display in my terminal. once I realized this I decided to go do it myself for fun. [eevee's toolkit](https://github.com/eevee/porigon-z) for data extraction can probably get those icon-sized sprites too, but I didn't bother checking.

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
- Fire Red
- Leaf Green
- Ruby
- Sapphire
- Emerald
- Heart Gold
- Soul Silver
- Diamond
- Pearl
- Platinum

Info (priority list):
- box sprites
- front sprites
- pokedex prose description
- pokedex attributes (height/weight)
- type/stats
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
- cli
  - hyphen option completion, otherwise name/number(id) and gender/form for second positional arg
