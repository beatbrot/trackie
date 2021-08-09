PREFIX=/usr/local
BINDIR=$(PREFIX)/bin
MANDIR=$(PREFIX)/share/man

trackie:
	cargo build --release

install: trackie
	install -D target/release/trackie $(DESTDIR)$(BINDIR)/trackie
	install -D doc/trackie.1          $(DESTDIR)$(MANDIR)/man1/trackie.1

uninstall:
	$(RM) $(DESTDIR)$(BINDIR)/trackie
	$(RM) $(DESTDIR)$(MANDIR)/man1/trackie.1

clean:
	cargo clean

.PHONY: clean install uninstall
