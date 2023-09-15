#include "header.h"

void init(int x, int* y)
{
   printf("***C function init x=%d, y=%d***\n", x, *y);
}

int destroy(struct small_data *data, int i)
{
    printf("i = %d\n", i);
    printf("first name = %s, last name = %s\n", data->first_name, data->last_name);
}
