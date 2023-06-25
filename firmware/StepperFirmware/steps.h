struct Steps {
    int32_t x;
    int32_t y;
    int32_t z;
    int32_t e;
};

Steps target_steps = {0, 0, 0, 0};
Steps pending_steps = {0, 0, 0, 0};
uint8_t stall_bitflag = 0b0000;
