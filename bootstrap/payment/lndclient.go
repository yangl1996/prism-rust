package main

import (
	"fmt"
	"io/ioutil"
	macaroon "gopkg.in/macaroon.v2"
	"github.com/lightningnetwork/lnd/lncfg"
	"github.com/lightningnetwork/lnd/lnrpc"
	"github.com/lightningnetwork/lnd/macaroons"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials"
	"os"
	"github.com/lightningnetwork/lnd/routing"
        "golang.org/x/net/context"

)

const tlsPath = "/root/.lnd/tls.cert"
const macPath = "/root/.lnd/data/chain/bitcoin/simnet/admin.macaroon"
const rpcPort = "10009"
const rpcPath = "localhost:10009"

func addInvoice (lnd lnrpc.LightningClient, amt int64) (string, error) {
	invoice := &lnrpc.Invoice{
		Value: amt,
	}
	resp, err := lnd.AddInvoice(context.Background(), invoice)
	if err != nil {
		return "", err
	}
	return resp.PaymentRequest, nil
}

func sendPayment (lnd lnrpc.LightningClient, payreq string) (*lnrpc.SendResponse, error) {
	sendReq := &lnrpc.SendRequest{
		PaymentRequest: payreq,
		SpiderAlgo:     routing.ShortestPath,
	}
	payresp, err := lnd.SendPaymentSync(context.Background(), sendReq)
	return payresp, err
}

func getLNDClientConn() *grpc.ClientConn {
	// load TLS sert
	creds, err := credentials.NewClientTLSFromFile(tlsPath, "")
	if err != nil {
		fatal(err)
	}

	// Create a dial options array.
	opts := []grpc.DialOption{
		grpc.WithTransportCredentials(creds),
	}

	// load macaroon
	macBytes, err := ioutil.ReadFile(macPath)
	if err != nil {
		fatal(fmt.Errorf("unable to read macaroon path (check "+
		"the network setting!): %v", err))
	}

	mac := &macaroon.Macaroon{}
	if err = mac.UnmarshalBinary(macBytes); err != nil {
		fatal(fmt.Errorf("unable to decode macaroon: %v", err))
	}

	// Now we append the macaroon credentials to the dial options.
	cred := macaroons.NewMacaroonCredential(mac)
	opts = append(opts, grpc.WithPerRPCCredentials(cred))

	// We need to use a custom dialer so we can also connect to unix sockets
	// and not just TCP addresses.
	opts = append(
		opts, grpc.WithDialer(
			lncfg.ClientAddressDialer(rpcPort),
		),
	)
	conn, err := grpc.Dial(rpcPath, opts...)
	if err != nil {
		fatal(fmt.Errorf("unable to connect to RPC server: %v", err))
	}

	return conn
}

func getLNDClient() (lnrpc.LightningClient, func()) {
	conn := getLNDClientConn()

	cleanUp := func() {
		conn.Close()
	}

	return lnrpc.NewLightningClient(conn), cleanUp
}

func fatal(err error) {
	fmt.Fprintf(os.Stderr, "[spider] %v\n", err)
	os.Exit(1)
}

