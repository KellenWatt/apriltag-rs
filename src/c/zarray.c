#include <apriltag/apriltag.h>
#include <apriltag/common/image_u8.h>
#include <apriltag/common/zarray.h>

int zarray_size__extern(zarray_t* za) {
    return zarray_size(za);
}

void zarray_get__extern(const zarray_t* za, int idx, void *p) {
    zarray_get(za, idx, p);
}

void zarray_destroy__extern(zarray_t* za) {
    zarray_destroy(za);
}

zarray_t* apriltag_detector_detect__extern(apriltag_detector_t* det, image_u8_t* img) {
    int width = img->width;
    printf("What C is seeing:\n");
    for(int i=0; i<width; i++) {
        printf("%d ", img->buf[i]);
    }
    printf("\n");
    return apriltag_detector_detect(det, img);
}
