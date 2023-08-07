struct small_data {
  char char_data;
  short short_data;
  int int_data;
};

void init(struct small_data *data, char x)
{
    data->char_data = x;
    data->short_data = x;
    data->int_data = x;
}

int main()
{
    struct small_data data;
    init(&data);
    return 0;
}