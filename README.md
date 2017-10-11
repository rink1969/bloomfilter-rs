# bloomfilter-rs
## 缘起
用Rust有一段时间了，但感觉还是没有捕捉住Rust的最佳实践，摸不清楚Rust到底最适合什么样的编程范式。

说是命令式吧，所有权搞得人很烦，老是得clone，没有C语言那么简单直接。

说面向对象吧，引用的生命周期又搞得人很烦。而且struct里面一个成员的引用，就把整个结构体都占住了，感觉特别反OO。

最后就是函数式了。其实Rust还是有不少类似函数式语言的语法。

看了 http://science.raphael.poss.name/rust-for-functional-programmers.html 感觉跟Haskell都能对应的上，就想试试函数式的风格。

刚好项目需要一个bloomfilter，就想参照 http://book.realworldhaskell.org/read/advanced-library-design-building-a-bloom-filter.html 写一个。

## 结论
先说结论吧，Rust一点都不函数式，充其量只是有点类似函数式的语法糖。都有变量了，谁还费那劲去搞什么monad，整个思路都不一样了。

可能会跟ocaml更像一点？难怪ocaml比Hashkell之类的要流行的多，原来就是上古时代的Rust。这个结论待确认，需要去多了解一下ocaml。

## bloomfilter
进入正题。bloomfilter就不多介绍了，比较简单，网上也有很多文章介绍了。

重点是如何实现一组高效高质量的Hash算法。因为每一个值的插入和查询都要算多次Hash，所以性能瓶颈明显就在这里。

也参考了一些开源的bloomfilter实现，其实大家都是在Hash算法上玩花样。

高效的实现，基本都是只要两个Hash算法，算出两个Hash值之后，通过线性组合，得出更多的Hash值。

本项目也是借鉴了Haskell那本书里面介绍的方法，包括 SpookyV2 这个Hash算法，也是那本书里面使用的Hash算法的最新版本。

这个Hash算法牛的地方在于，Hash的质量很高，而且一次Hash就可以得到两个u64的Hash值，再通过线性组合，就可以得到一组Hash值了。

最终测试结果，比目前cargo仓库里的那个bloomfilter还要快10~20%。
