struct small_data {
  char char_data;
  short short_data;
  int int_data;
};

void init(struct small_data *data)
{
    data->char_data = 0;
    data->short_data = 0;
    data->int_data = 0;
}

int main()
{
    struct small_data data;
    init(&data);
    return 0;
}