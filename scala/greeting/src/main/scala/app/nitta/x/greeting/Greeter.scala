package app.nitta.x.greeting

import java.nio.file.Files
import org.apache.lucene.store.MMapDirectory

object Greeter extends App {
  val dirPath = Files.createTempDirectory("lucene-index-")
  val dir = new MMapDirectory(dirPath)
  println("Hello, world!")
  dir.close()
  Files.deleteIfExists(dirPath)
}
