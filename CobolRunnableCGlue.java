import jp.osscons.opensourcecobol.libcobj.common.*;
import jp.osscons.opensourcecobol.libcobj.call.*;
import jp.osscons.opensourcecobol.libcobj.data.*;

abstract public class CobolRunnableCGlue implements CobolRunnable {
    @Override
    public void cancel() {
    }

    @Override
    public boolean isActive() {
        return false;
    }

    byte[] storageToByteArray(CobolDataStorage storage, int size) {
        return null;
    }

    byte storageToByte(CobolDataStorage storage) {
        return 0;
    }

    short storageToShort(CobolDataStorage storage) {
        return 0;
    }

    int storageToInt(CobolDataStorage storage) {
        return 0;
    }
}