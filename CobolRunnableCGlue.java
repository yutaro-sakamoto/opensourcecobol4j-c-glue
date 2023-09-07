import jp.osscons.opensourcecobol.libcobj.common.*;
import jp.osscons.opensourcecobol.libcobj.call.*;
import jp.osscons.opensourcecobol.libcobj.data.*;
import java.nio.ByteBuffer;

abstract public class CobolRunnableCGlue implements CobolRunnable {
    @Override
    public void cancel() {
    }

    @Override
    public boolean isActive() {
        return false;
    }

    byte[] storageToByteArray(CobolDataStorage storage, int size) {
        return storage.getByteArray(0, size);
    }

    byte storageToByte(CobolDataStorage storage) {
        return storage.getByte(0);
    }

    short storageToShort(CobolDataStorage storage) {
        return ByteBuffer.wrap(storage.getByteArray(0, 2)).getShort();
    }

    int storageToInt(CobolDataStorage storage) {
        return ByteBuffer.wrap(storage.getByteArray(0, 4)).getInt();
    }

    void bytesToStorage(CobolDataStorage storage, byte[] bytes) {
        storage.setBytes(bytes);
    }

    void byteToStorage(CobolDataStorage storage, byte b) {
        storage.setByte(b);
    }

    void shortToStorage(CobolDataStorage storage, short s) {
        storage.setBytes(ByteBuffer.allocate(2).putShort(s).array());
    }

    void intToStorage(CobolDataStorage storage, int i) {
        storage.setBytes(ByteBuffer.allocate(4).putInt(i).array());
    }
}