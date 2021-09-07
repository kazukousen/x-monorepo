package app.nitta.x.greeting

import org.apache.lucene.store.RAMDirectory

object Greeter extends App {
  val dir = new RAMDirectory
  println("Hello, world!")
}
