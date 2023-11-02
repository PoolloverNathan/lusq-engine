import java.nio.file.Files;
class LoaderRef;

external fun compile(program: String): ByteArray

fun main(args: Array<String>) {
  val lib = Files.createTempFile("liblusque-", ".so")
  Files.copy(LoaderRef::class.java.getResource("/liblusque.so"), lib)
  System.loadLibrary(lib)
  println(compile(""))
}