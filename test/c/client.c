#include <stdio.h> 
#include <sys/socket.h> 
#include <arpa/inet.h> 
#include <unistd.h> 
#include <string.h> 
#define PORT 8080

int Send() {
    printf("xxxxxx1\n");

    char *hello = "Hello from client";
    struct iovec iov[1];
    iov[0].iov_base = hello;
    iov[0].iov_len = strlen(hello);

    struct msghdr mh;
    mh.msg_name = hello;
    mh.msg_namelen = strlen(hello);
    mh.msg_iov = iov;
    mh.msg_iovlen = 1;
    mh.msg_control = hello;;
    mh.msg_controllen = strlen(hello);
    mh.msg_flags = 0;

    printf("xxxxxx2\n");

    int sock = 0;
    if ((sock = socket(AF_INET, SOCK_STREAM, 0)) < 0)
    {
        printf("\n Socket creation error \n");
        return -1;
    }

    printf("xxxxxx3\n");
    int ret = sendmsg(sock, &mh, 0);
    printf("output is %d\n", ret);
    return ret;
}

int main(int argc, char const *argv[]) 
{
    //printf("xxxxxx\n");
    //return Send();

    int sock = 0, valread;
    struct sockaddr_in serv_addr; 
    char *hello = "Hello from client"; 
    char buffer[1024] = {0}; 
    if ((sock = socket(AF_INET, SOCK_STREAM, 0)) < 0) 
    { 
        printf("\n Socket creation error \n"); 
        return -1; 
    }

    struct sockaddr_in sa;
    int sa_len;
    sa_len = sizeof(sa);
    printf("sa_len: %d\n", sa_len);
    printf("sock is %d\n", sock);
    if (getsockname(sock, &sa, &sa_len) == -1) {
          perror("getsockname() failed");
          return -1;
    }
    printf("Local IP address is: %s\n", inet_ntoa(sa.sin_addr));
    printf("Local port is: %d\n", (int) ntohs(sa.sin_port));

    int ret = 0;
    if (getpeername(sock, &sa, &sa_len) == -1) {
              printf("errorno: %d\n", errno);
              perror("getsockname() failed");
              return -1;
    }
    printf("Remote IP address is: %s\n", inet_ntoa(sa.sin_addr));
    printf("Remote port is: %d\n", (int) ntohs(sa.sin_port));

    printf("start to connect \n");
    sleep(1);

    serv_addr.sin_family = AF_INET;
    serv_addr.sin_port = htons(PORT); 
       
    // Convert IPv4 and IPv6 addresses from text to binary form 
    //if(inet_pton(AF_INET, "127.0.0.1", &serv_addr.sin_addr)<=0)
    if(inet_pton(AF_INET, "172.16.1.6", &serv_addr.sin_addr)<=0)
    { 
        printf("\nInvalid address/ Address not supported \n"); 
        return -1; 
    } 
   
    if (connect(sock, (struct sockaddr *)&serv_addr, sizeof(serv_addr)) < 0) 
    { 
        printf("\nConnection Failed \n"); 
        return -1; 
    }

    printf("Accept Remote IP address is: %s\n", inet_ntoa(serv_addr.sin_addr));
    printf("Accept Remote port is: %d\n", (int) ntohs(serv_addr.sin_port));

    // struct sockaddr_in sa;
    // int sa_len;
    // sa_len = sizeof(sa);
    if (getsockname(sock, &sa, &sa_len) == -1) {
          perror("getsockname() failed");
          return -1;
    }
    printf("Local IP address is: %s\n", inet_ntoa(sa.sin_addr));
    printf("Local port is: %d\n", (int) ntohs(sa.sin_port));

    if (getpeername(sock, &sa, &sa_len) == -1) {
              perror("getsockname() failed");
              return -1;
    }
    printf("Remote IP address is: %s\n", inet_ntoa(sa.sin_addr));
    printf("Remote port is: %d\n", (int) ntohs(sa.sin_port));

    valread = read(sock , buffer, 1024);
    printf("%s\n",buffer );
    int i = write(sock , hello , strlen(hello));
    printf("client: Hello message sent %d bytes\n", i);

    sleep(1);
    i = send(sock , hello , strlen(hello) , 0 );
    printf("client: Hello message sent %d bytes\n", i);
    valread = recvfrom(sock , buffer, 1024, 0, &sa, &sa_len);
    printf("%d recvfrom: %s\n", valread, buffer);
    printf("recvfrom: Remote IP address is: %s\n", inet_ntoa(sa.sin_addr));
    printf("recvfrom: Remote port is: %d\n", (int) ntohs(sa.sin_port));

    struct iovec iov[3];
    iov[0].iov_base = hello;
    iov[0].iov_len = strlen(hello);
    iov[1].iov_base = hello;
    iov[1].iov_len = strlen(hello);
    iov[2].iov_base = hello;
    iov[2].iov_len = strlen(hello);

    struct msghdr mh;
    mh.msg_name = 0;
    mh.msg_namelen = 0;
    mh.msg_iov = iov;
    mh.msg_iovlen = 3;
    mh.msg_control = NULL;
    mh.msg_controllen = 0;
    mh.msg_flags = 0;

    for (i=0; i< 10; i++) {
        int n = sendmsg(sock, &mh, 0);
        printf("client: sendmsg sent %d bytes\n", n);
    }

    return 0;
} 
