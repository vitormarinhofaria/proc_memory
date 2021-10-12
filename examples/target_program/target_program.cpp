#include <iostream>

template<typename T>
T* allocate_mem(size_t address);

int main(int argc, char** argv)
{
    size_t addr = 0x00007FF49E872000;
    uint64_t* number = allocate_mem<uint64_t>(addr);
    *number = 42;
    std::cout << number << " - " << *number << '\n';
    
    std::cout << "Press ENTER to update value\n";
    std::cin.get();
    
    std::cout << number << " - " << *number << '\n';
    
    std::cout << "Press ENTER to exit\n";
    std::cin.get();
}

#ifdef _WIN32
#include <Windows.h>
template<typename T>
T* allocate_mem(size_t address)
{
    auto pointer = VirtualAlloc((LPVOID)address, sizeof(T), MEM_COMMIT, PAGE_READWRITE); 
    return (T*)pointer;
}
#else
#include <sys/mman.h>
#include <fcntl.h>
template<typename T>
T* allocate_mem(size_t address)
{
    return mmap((void*)address, sizeof(T), PROT_WRITE | PROT_READ, 0, open("/dev/null", O_RDWR), 0);
}
#endif
