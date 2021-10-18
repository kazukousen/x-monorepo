package main

import (
	"context"
	"log"
	"net"
	"os"
	"os/signal"
	"syscall"

	"google.golang.org/grpc"
	"google.golang.org/grpc/reflection"

	detailv1 "github.com/kazukousen/x-monorepo/protos/gen/proto/go/detail/v1"
)

func main() {

	svc := &service{}

	s := grpc.NewServer()
	detailv1.RegisterDetailServiceServer(s, svc)
	reflection.Register(s)

	lis, err := net.Listen("tcp", ":8080")
	if err != nil {
		log.Fatalf("unable to listen: %v", err)
	}

	go func() {
		if err := s.Serve(lis); err != nil {
			log.Fatalf("failed to serve: %v", err)
		}
	}()

	stop := signalHandler()
	<-stop

	s.GracefulStop()
}

func signalHandler() <-chan struct{} {
	stop := make(chan struct{}, 0)

	go func() {
		quit := make(chan os.Signal, 2)
		signal.Notify(quit, os.Interrupt, syscall.SIGTERM, syscall.SIGINT)

		log.Printf("received signal, wait shutting down: %s", <-quit)
		close(stop)
		log.Fatalf("received twice signal, directly exit: %s", <-quit)
	}()

	return stop
}

type service struct {
}

func (s service) GetDetails(ctx context.Context, req *detailv1.GetDetailsRequest) (*detailv1.GetDetailsResponse, error) {
	ret := make([]*detailv1.Hotel, len(req.HotelIds))

	for i, id := range req.HotelIds {
		ret[i] = &detailv1.Hotel{
			Id: id,
		}
	}

	return &detailv1.GetDetailsResponse{
		Hotels: ret,
	}, nil
}

var _ detailv1.DetailServiceServer = (*service)(nil)
